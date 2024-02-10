# Exploiting the unsoundness, method 1

For context:

```rust ,ignore
pub
struct Animaterium<'soul, Body : ForLt> {
    _soul: PhantomInvariant<'soul>,
    carcass: Body::Of<'static>,
}

pub
fn soul_split<'soul, Body : ForLt>(
    value: Body::Of<'soul>,
) -> Animaterium<'soul, Body>
{
    Animaterium {
        _soul: <_>::default(),
        carcass: unsafe {
            ::core::mem::transmute::<Body::Of<'soul>, Body::Of<'static>>(value)
        },
    }
}

impl<'soul, Body : ForLt> Animaterium<'soul, Body> {
    pub
    fn into_inner(self: Animaterium<'soul, Body>)
      -> Body::Of<'soul>
```

Let's start by quoting the first two sets of hints from the previous section:

> 1. <details open><summary>Click to reveal</summary>
>
>      - The main `unsafe` operation here is the `transmut`ing of `'soul -> 'static` lifetimes in the `fn soul_split()` construction.
>
>      - The "reasoning" behind the so-claimed "SAFETY" of that operation was the lack of exposure of the inaccurate `Body::of::<'static>` to "the outside world" / to public API.
>
>      - Maybe that claim was too bold and some exposure slipped through the cracks / leaked?
>
>    </details>
>
> 1. <details open><summary>Click to reveal</summary>
>
>      - Did someone say _API leakage_?
>
>      - What typical API property other than the pure `fn`s written above may be affected by the choice of field types?
>
>    </details>

And that last question hits the nail on the head. That property, and that's something generally useful to know about Rust when dealing with, _e.g._, `-> impl Trait`s or `async fn`s, are **auto-traits**.

```rs ,ignore
/// 1. This type represents `Body::Of<'soul>`.
pub
struct Animaterium<'soul, Body : ForLt> {
    /// 2. This field is `Send + Sync`.
    _soul: PhantomInvariant<'soul>,

    /// 3. This field loses `Send/Sync` when `Body::Of<'static>` does.
    carcass: Body::Of<'static>,
}
```

Thus, `Animaterium<'soul, Body>`, which is supposed to be a `Body::Of<'soul>` value, in practice, is "as `Send/Sync`" as `Body::Of<'static>`.

## Exploit

Remember when I mentioned "edge cases" of unsoundness. Well, brace yourselves, for the exploit presented herein is indeed _very_ contrived. And in fact, it will involve an instance of `unsafe` on its end, so we could end up arguing semantics. But to be fair, its usage of `unsafe` is quite legitimate, actually, and thus does showcase the existence of an exploit.

We need:

 1. `Body::Of<'static> ≠ Body::Of<'soul>`, thus, we need a type expression which does depend on a lifetime;

 1. and in so doing, which happens to showcase a _change_ of one of its auto-traits, _e.g._, `Send`;

 1. with such auto-trait playing a key role w.r.t. soundness.

### The API library to exploit

For starters, let's consider the following snippet where improper `Send`ness would cause unsoundness:

```rust
use ::core::cell::Cell as Mutable;

pub
struct PerThreadSingleton {
    _not_send: ::core::marker::PhantomData<*mut ()>,
}

thread_local! {
    static EXISTS: Mutable<bool> = const { Mutable::new(false) };
}

impl PerThreadSingleton {
    fn new() -> Option<PerThreadSingleton> {
        EXISTS.with(|already_exists| {
            if already_exists.get() {
                None
            } else {
                already_exists.set(true);
                Some(PerThreadSingleton {
                    _not_send: <_>::default(),
                })
            }
        })
    }
}

// Usage:
PerThreadSingleton::new().map(|singleton| {
    // use `singleton` here…
});
```

From here, we have a type:
  - which, by virtue of not being `Send`, cannot cross thread boundaries in an owned or exclusive fashion,

  - which locks a thread from being able to produce new instances once one has been created;

  - which implements no `Clone` or way to be duplicated.

Thus, it is literally impossible to call a function which would take _two_ `PerThreadSingleton` instances as args.

That is, the following function can soundly be exposed:

```rust ,ignore
pub
fn two_instances(
    _a: PerThreadSingleton,
    _b: PerThreadSingleton,
) -> !
{
    // This is so impossible, that if a code path were to somehow hit this, then
    // it would be equivalent to having an instance of an uninhabited `enum`.
    // This can simply not be, and we decide to help the optimizer a bit by
    // telling it so:
    unsafe {
        // UB if reached!
        ::core::hint::unreachable_unchecked()
    }
}
```

Now, for the sake of transitioning back to our original exploit, let's imagine tweaking the `fn new()` constructor a little bit: instead of a classic constructor, let's feature a _scoped/callback API_ instead:

```rust ,ignore
impl PerThreadSingleton {
    pub
    fn with_new(scope: impl FnOnce(PerThreadSingleton))
    {
        let yield_ = scope;
        EXISTS.with(|already_exists| {
            if already_exists.get().not() {
                already_exists.set(true);
                yield_(PerThreadSingleton {
                    _not_send: <_>::default(),
                });
            }
        })
    }
}

// Usage:
PerThreadSingleton::with_new(|singleton| {
    // use `singleton` here…
})
```

And despite the change in ctor signature, the key "unicity"/singleton property remains unscathed, so we can still be _soundly_ featuring the `two_instances()` function above.

___

Now, a scoped API is usually provided to do some _cleanup_ after the callback (a cleanup which, in real code, may be so critical that it cannot be tied to some `Drop` glue of a type _given_ to the caller, lest they `mem::forget()` it. For more context, _c.f._ the "Leakpocalypse" or how convoluted the `thread::scope()` API of the stdlib has to be).

  - When the cleanup is so critical, whatever is yielded to the callback should not escape the scope of the callback.

  - As of now, a caller could do:

    ```rust ,ignore
    fn my_new() -> Option<PerThreadSingleton> {
        let mut smuggling_channel = None;
        PerThreadSingleton::with_new(|it| smuggling_channel = Some(it));
        return smuggling_channel; // escaped!
    }
    ```

      - And in fact, every language but Rust suffers from this problem!

Rust, however, has a powerful tool to guard against this: lifetimes! It turns out we can actually express a lifetime which semantically acts as kind of the lifetime of the scope, and from there, `my_new()` can be rejected:

```rust ,ignore
struct PerThreadSingleton<'scope> {
    /* previous fields… */

    // added:
    _scope: PhantomData<&'scope ()>, // important that this not be contravariant;
                                     // in practice, it can be wise to make it
                                     // non-covariant as well, i.e., invariant,
                                     // but for the sake of teachability we're
                                     // gonna live on the edge, here.
}

                    // we don't care about this one, but _something_ had to be written.
                    // vvvv
impl PerThreadSingleton<'_> {
    pub                                    // Added! This is doing the magic.
    fn with_new(                           //             👇
        scope: impl for<'scope> FnOnce(PerThreadSingleton<'scope>),
    )
    {
        let yield_ = scope;
        EXISTS.with(|already_exists| {
            if already_exists.get().not() {
                already_exists.set(true);
                yield_(PerThreadSingleton {
                    _not_send: <_>::default(),
                });
                already_exists.set(false); // 👈 allowing us to soundly add this!
            }
        })
    }
}
```

  - I'm going to skip properly explaining how this `for<'scope>` lifetime in `fn with_new()` achieves to represent that of the scope of the callback, or rather, one which cannot escape it. Re-read the [`sort_by_key()` section](motivating-example-10-explain.md) for more info.

And now we've finally gotten a type, `PerThreadSingleton<'_>`, which can only be instantiated through this scoped API, **thereby yielding non-`'static` instances** (this is going to be important Soon™), and which cannot be `Send` lest a function such as `fn two_instances()` become problematic.

```rust ,ignore
{{#include may_dangle_oibit_exploit.rs:two-instances}}
```

And now, the missing piece/ingredient: making `PerThreadSingleton<'static> : Send` 😈.

Indeed, it is impossible to produce such an instance (since we have replaced `fn new()` with `fn with_new()`). So it's a type with an unreachable API, therefore a _useless_ API; _a fortiori_, a _harmless_ API:

```rust ,ignore
{{#include may_dangle_oibit_exploit.rs:impl-send}}
```
___

With all that having been laid out, it is time for:

### The exploit of such API

```rust ,ignore
{{#include may_dangle_oibit_exploit.rs:exploit}}
```

**[Full snippet Playground](https://play.rust-lang.org/?version=nightly&mode=debug&edition=2021&gist=0a275067cad59df80213e435691fe92b)**

<details><summary>Click here to fiddle with the full snippet inline</summary>

```rust ,edition2018,editable
{{#include may_dangle_oibit_exploit.rs:all}}
```

</details>

___

## Epilogue

  - I did warn the snippet / API library was gonna be contrived! But it has the _right_ of doing everything it does.

  - We can thus palliate this auto-trait problem by adding an unconditional `not_send_nor_sync: PhantomData<*mut ()>` field to our `Animaterium<'soul, Body>` definition;

  - and yet… _even then_ our `Animaterium<'soul, Body>` will be unsound for certain choices of `Body` 😱, as we will see in the very next section.

  - what a nice _segue_ to it, is it not?

    <img
        src="https://gist.github.com/assets/9920355/78b745ca-c534-4641-b7a2-4da68aa754d7"
        title = "totally not a segue"
        alt = "a segway"
        height = "250px"
    />
