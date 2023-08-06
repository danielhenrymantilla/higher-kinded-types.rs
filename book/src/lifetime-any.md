# Motivation: lifetime-infected `dyn Any` erasure

<!-- toc -->

### A brief introduction about `Any : 'static`

Depending on your Rust needs, you may have needed to use type erasure, using
`dyn Trait`s, and in the more extreme case, you might not have had a single
`Trait` behind which to hide your types other than `Any`, the API that consists
of `dyn`amic/at-runtime type identification, which thereby allows guessing the
originally erased type, so as to undo the `dyn` erasure if we get it right
(called type downcasting).

The issue in question, here, is with lifetimes:

  - `dyn Any` is a type with no lifetime within it (it is `: 'static`), so any lifetime information
    present in the original type before erasure is not present in the static/compile-time/type-level
    information of `dyn Any`.

  - but lifetimes are purely a compile-time construct!

      - (technically you can implement a Rust compiler that skips
        borrow-checking and directly tries to compile the (hopefully correct) code into machine
        code, and if you do that then your compiler will be able to fully[^ignoring_lifetimes]
        ignore lifetimes).

[^ignoring_lifetimes]: `for<'a, 'b>`-arity, on the other hand, does play a role and is able to lead to different monomorphizations, though. Otherwise `fn(&str) -> &str` and `fn(&str) -> &'static str)`, which are both `: 'static` would be mixed up by `Any`.

  - this means that it won't be possible to query back that lost lifetime information within the
    runtime/`dyn`amic self-type-identification machinery.

So, because of this, types are not allowed to carry lifetime information/restrictions within them, in
order for them be soundly erasable to `dyn Any`: this property is achieved thanks to the notorious
`: 'static` bound on `Any`.

But this, then, leads to the following legitimate question:

### Is it possible to have a type-erasure mechanism allowing downcasts to lifetime-infected types?

Based on the previous section you may be tempted to say no, except for one important detail around
which we can try to work:

> `dyn Any` is a type with no lifetime within it (it is `: 'static`)

What about `dyn Any + 'lt`, then?

As a thought experiment, let's imagine having a `&'static i32`, such as `&42` (thanks to [`static`
promotion](https://rust-lang.github.io/rfcs/1414-rvalue_static_promotion.html)).

This is a `: 'static` type, so it can be coërced to a `dyn Any`, with the `TypeId::of::<&i32>()`.

Now say you have a `&'u i32`. What if, with a lil unsafe, we did the following:

```rust ,ignore
use ::core::any::Any;

/// Is this sound?
fn we_do_a_lil_unsafe<'u>(
    r: &'u i32,
) -> Box<dyn 'u + Any>
{
    // Safety: this `'static` is immediately erased to `dyn 'u + …`, so the
    // resulting entity won't be usable beyond `'u`.
    let r: &'static i32 = unsafe { ::core::mem::transmute(r) };
    Box::new(r)
}
```

What do you think of this snippet? Is it sound?

Well, the following compiling just fine is a bit problematic, is it not?

```rust ,edition2018
use ::core::any::Any;

fn wOnT_bE_uSaBlE_bEyOnD<'u>(
    r: Box<dyn 'u + Any>
) -> Box<dyn 'static + Any>
{
    r
}
```

  - Indeed, `: 'static` is a super-bound of `Any`, meaning that whenever something is `: Any`,
    then it also is `: 'static`! So a `Box<dyn 'u + Any>` is actually a
    `Box<dyn 'u + 'static + Any>`, which, in turn, is a `Box<dyn 'static + Any>` since `: 'a + 'b`
    is equivalent to `: union('a, 'b) ~ max('a, 'b) = 'static when 'b = 'static` (people often
    trip up on this, since they're so used to seeing `+ 'u` as a max-bound of usability of items,
    when it's actually _a lower bound_.

    Same as with `FnMut()` and `FnOnce()`, for instance: whilst
    an arbitrary `F : FnOnce()` may only be callable once (conservative assumption _barring extra
    information_), a `F : FnOnce() + FnMut()` is not an oxymoron, but just a plain `F : FnMut()`.
    It's the same with `: 'region_of_usability`)

    In other words, the very `: 'static` which had been slapped onto `Any` for soundness, is
    actually the one giving, here, a capability which makes `dyn Any`s "too strong", and thus,
    unsound-prone; for custom shenanigans such as this one at least.

    <div style="text-align: center;"><img
        src = "https://user-images.githubusercontent.com/9920355/258630076-ac1917fb-ac61-407c-a000-01e68dc51974.png"
        height = "150px"
        title = "ironic"
        alt = "the irony is rich"
    /></div>

Hum, this will require some extra hoops then. Let's try to rewrite `Any` but without the mandatory
`: 'static` available to `dyn`s:

```rust ,edition2018
use ::core::any::TypeId;

// for reference, `Any`'s definition:
#[cfg(feature = "🎶 if I were a core 🎶")]   /* https://youtu.be/4U_RvUYINpo */
mod real_any {
    pub
    trait Any : 'static {
        fn type_id(&self) -> TypeId;
    }
    impl<T : ?Sized + 'static> Any for T {
        fn type_id(&self) -> TypeId {
            TypeId::of::<T>()
        }
    }
}

pub
trait MyAny : seal::StaticSealed {
    fn type_id(&self) -> TypeId;
}

impl<T : ?Sized + 'static> MyAny for T {
    fn type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }
}

// Main trick: this, much like `Any`'s own `: 'static`, makes `: 'static` a
// mandatory step to be `MyAny`, but the big difference is that despite
// the requirement, we don't get the reverse implication: as far as Rust is
// concerned, there could exist `T : MyAny` for which `T : 'static` would
// not hold!
mod seal {
    pub trait StaticSealed {}
    impl<T : ?Sized + 'static> StaticSealed for T {}
}

// --------------------------------

// from there, the usual downcasting shenanigans:
impl dyn '_ + MyAny {
    pub
    fn is<T : 'static>(&self) -> bool {
        self.type_id() == TypeId::of::<T>()
    }

    pub
    fn downcast_ref<T : 'static>(&self) -> Option<&T> {
        self.is::<T>().then(|| unsafe {
            &*(self as *const Self as *const T)
        })
    }
}

/// Is this sound?
fn we_do_a_lil_unsafe<'u>(
    r: &'u i32,
) -> Box<dyn 'u + MyAny>
{
    // Safety: this `'static` is immediately erased to `dyn 'u + …`,
    // so the resulting entity won't be usable beyond `'u`.
    let r: &'static i32 = unsafe { ::core::mem::transmute(r) };
    Box::new(r)
}
```

And now to check that the `'u` is an effective bound:

```rust ,compile_fail
fn lets_see_the_region_of_usability<'u, 'tell_me>(
    input: Box<dyn 'u + MyAny>
) -> impl 'tell_me + Sized
{
    input
}
```

yields:

```rust ,ignore
error: lifetime may not live long enough
  --> src/lib.rs:67:5
   |
63 | fn lets_see_the_region_of_usability<'u, 'tell_me>(
   |                                     --  -------- lifetime `'tell_me` defined here
   |                                     |
   |                                     lifetime `'u` defined here
...
67 |     input
   |     ^^^^^ function was supposed to return data with lifetime `'tell_me` but it is returning data with lifetime `'u`
   |
   = help: consider adding the following bound: `'u: 'tell_me`
```

So any `'tell_me` so that a `Box<dyn 'u + MyAny>` may be used, has to satisfy `'u ⊇ 'tell_me`,
_i.e._, the biggest such one is `'u` itself, _i.e._, a `Box<dyn 'u + MyAny>` is very much
_not usable beyond `'u`_.

So all is good, right?

<img
    src="https://user-images.githubusercontent.com/9920355/243207321-63ad631e-8fb6-458e-8aa8-6e44f868386d.png"
    alt="right?"
    title="right?"
    height="200px"
/>

```rust ,ignore
fn uh_oh<'u>(
    r: Box<dyn 'u + MyAny>,
) -> &'static i32
{
    *r.downcast_ref::<&i32>().unwrap()
}
```

Indeed, whilst returning a properly `'u`-bounded entity was _necessary_ for soundness, it was not
_sufficient_: a `'u`-bounded entity may still allow certain APIs to extract non-`'u`-bounded stuff
out of it!

And in this instance, the very API allowing downcasts was the culprit:

```rust ,ignore
impl<'u> dyn 'u + MyAny {
    pub
    fn downcast_ref<'r, R : 'static>(
        self: &'r (dyn 'u + MyAny),
    ) -> Option<&'r R>
    {
        self.is:<R>().then(|| unsafe {
            &*(self as *const Self as *const R)
        })
    }
}
```

which, for `R = &'static i32`, becomes:

```rust ,ignore
impl<'u> dyn 'u + MyAny {
    pub
    fn downcast_ref<'r, &'static i32>(
        self: &'r (dyn 'u + MyAny),
    ) -> Option<&'r &'static i32>
    {
        (self.type_id() == TypeId::of::<&'static i32>()).then(|| unsafe {
            &*(self as *const Self as *const T)
        })
    }
}
```

Notice how we end up with a check from `self.type_id()` (which returns `TypeId::of::<&'static i32>()`
for our constructed value), against the very same `TypeId::of::<&'static i32>()`.

The check passes, and we end up with a `&'r &'static i32`, with `'r` being the lifetime of the
`self: &'r (dyn 'u + MyAny)` receiver.

Which is properly `'r`-bounded and thus `'u`-bounded (since `'u ⊇ 'r`), but from which we can simply
extract the `&'static i32`, unbounded, by simple `*`-dereference.

Uh-oh.

> So our API is not sound _yet_.

Before tackling a more general fix to this problem, let's palliate it by replacing our
`downcast_ref` above with the following more limited API:

<details><summary>Click here to see intermediary steps</summary>

 1. We had:
    ```rust ,ignore
    impl<'u> dyn 'u + MyAny {
        pub
        fn downcast_ref<'r, R : 'static>(
            self: &'r (dyn 'u + MyAny),
        ) -> Option<&'r R>
        {
            self.is:<R>().then(|| unsafe {
                &*(self as *const Self as *const R)
            })
        }
    }
    ```

 1. From there, we narrow the generic `R` down to the `&'static T` shape:
    ```rust ,ignore
    impl<'u> dyn 'u + MyAny {
        pub
        fn downcast_ref<'r, T : 'static>(
            self: &'r (dyn 'u + MyAny),
        ) -> Option<&'r &'static T>
        {
            self.is:<&'static T>().then(|| unsafe {
                &*(self as *const Self as *const &'static T)
            })
        }
    }
    ```

 1. And finally:

</details>

```rust ,ignore
impl<'u> dyn 'u + MyAny {
    /* no more downcast_ref! 👈 */
    pub
    fn downcast_bounded_ref<'r, T : 'static>(
        self: &'r (dyn 'u + MyAny),
    ) -> Option<&'r &'u T>
    { //         👇   👆👆
        self.is::<&T>().then(|| unsafe {
            //                                👇
            &*(self as *const Self as *const &'static T) // : &'r &'static T
                                                         // : &'r &'u      T
        })
    }
}
```

  - The main difference is that the `TypeId` check now unconditionally targets `&T` references,
    rather than arbitrarily generic `T`s. In other words, if we name `R` the argument given to
    `TypeId::of`, we are restricting ourselves to `R = &T = &_` types.

    By doing this, we were then able to do another thing: rather than returning `-> &R` _i.e._,
    `-> &&'static T`, which was problematic, we instead returned `&&'u T`, thereby having restricted
    the problematic lifetime accordingly!

    As a matter of fact, the very reason to restrict `R` down to the `&'_ T` shape was to be able
    to, upfront, get access to a lifetime placeholder wherein we'd be able to manually replace the
    problematic `'static` with `'u`. This notion of replacing a classic generic parameter with
    a restricted shape so that a lifetime placeholder be exposed upfront ought to sound very similar
    to the [`sort_by_key_ref()` signature in our `sort_by_key()` chapter](
    ./explain-sort-by-lifetimes.md#solving-the-returned-borrow-problem).

Finally, we can simplify it down a bit by realizing the outer `&'r` is not playing any role here:
if we are to return `-> &'r &'u T`, we may as well return `&'u T`! Much like we return `-> bool`s
rather than `&bool`:

```rust ,ignore
impl<'u> dyn 'u + MyAny {
    /* no more downcast_ref! */
    pub
    fn downcast_bounded_ref<'r, T : 'static>(
        self: &'r (dyn 'u + MyAny),
    ) -> Option<&'u T>
    {
        self.is::<&T>().then(|| unsafe {
            &*(self as *const Self as *const &'static T) // : &'r &'static T
                                                         // : &'r &'u      T
        })
        .map(|r: &'r &'u T| -> &'u T { *r }) // for convenience
    }
}
```

**With this API, we finally got `we_do_a_lil_unsafe()` to become a sound API!**

```rust ,ignore
/// Sound!
pub
fn we_do_a_lil_unsafe<'u>(
    r: &'u i32,
) -> Box<dyn 'u + MyAny>
{
    // SAFETY:
    //  1. this `'static` is immediately erased to `dyn 'u + …`,
    //     so the resulting entity won't be usable beyond `'u`;
    //  2. the only way to extract this `&i32` back from a `dyn MyAny` is
    //     through `downcast_bounded_ref`, which also yields a `'u`-bounded &i32
    let r: &'static i32 = unsafe { ::core::mem::transmute(r) };
    Box::new(r)
}
```

So, until now, we've been attempting to `dyn Any`-erase a `&'u i32`, but _quid_ of other
`'u`-infected types?

#### How well does this generalize to another lifetime-infected type such as `Cell<&_>`?

Let's consider, now, for instance, the type `Cell<&'u i32>`:

```rust ,ignore
/// Is this sound?
pub
fn we_do_a_lil_unsafe_2<'u>(
    r: Cell<&'u i32>,
) -> Box<dyn 'u + MyAny>
{
    // SAFETY:
    //  1. this `'static` is immediately erased to `dyn 'u + …`,
    //     so the resulting entity won't be usable beyond `'u`;
    //  2. the only way to extract this `Cell<&i32>` back from a `dyn MyAny` is
    //     through `downcast_cell_ref`, which also yields a `'u`-bounded Cell<&i32>
    let r: Cell<&'static i32> = unsafe { ::core::mem::transmute(r) };
    Box::new(r)
}

impl<'u> dyn 'u + MyAny {
    fn downcast_cell_ref<'r, T : 'static>(
        self: &'r (dyn 'u + MyAny),
    ) -> Option<&'r Cell<&'u T>>
    {
        self.is::<Cell<&'static T>>().then(unsafe {
            &*(self as *const Self as *const Cell<&T>)
        })
    }
}
```

So, is this sound?

<img
    src="https://user-images.githubusercontent.com/9920355/243356995-0bfdfa9f-e2d8-4520-af76-cfd656724f8b.png"
    alt="well yes but actually no"
    title="well yes but actually no"
    height="200px"
/>

Indeed, as long as we were dealing with `&'u i32` kind of references, the only pitfall to be careful
with was ending up with a `&'new_u i32` wherein `'new_u` might be _bigger_ than `'u` (lifetime
_extension_ for references is unsound).

  - This is why my talk has been about (upper-)_bounding_ the `'new_u` so as to ensure that
    `'u : 'new_u` always hold.

But with `Cell<&'u i32>`, we have not only that problem (lifetime _extension_ being unsound), but we
also have _another constraint_:

  - lifetime _reduction_ behind a `Cell` is unsound too! (we say that `Cell<_>` is **non-covariant**
    (and since it is not contravariant either, we call this invariance)).

    For those unconvinced, see this:

    ```rust ,edition2018
    use ::core::cell::Cell as Mut; // <- just this should already scream non-covariance for `Cell`.

    fn unsound<'local>(
        s: &'local Mut<&'static str>,
    ) -> &'local Mut<&'local str>
    {
        // If `Cell`s were covariant, we'd be able to get this API.
        unsafe { ::core::mem::transmute(s) }
    }

    let mut s: &'static str = "static str";
    {
        let at_mut_static: &Mut<&'static str> = Mut::from_mut(&mut s);
        let local:      &'_ /* 'local */ str = &String::from("...");
        Mut::</* &'local str */>::set(
            unsound(at_mut_static), // : &'_ Mut<&'local str>
            local,                  // : &'local str
        );
    } // <- the `String` is dropped; `local` henceforth dangles.
    let _unrelated: String = "UB!".into();
    println!("{s}");
    ```

    Feel free to click on the <i class="fa fa-play play-button"></i> of that snippet. It features UB,
    since it will read a dangling pointer leading to a Use-After-Free, so the exact behavior is
    technically unpredictable. With that being said, I've written it in a way which made `{s}`,
    when I ran it, read the memory of the `_unrelated: String`, thereby printing `UB!`!

    ___

      - Anecdotical aside (feel free to skip it!)

        In fact, I had to cheat a bit in the `downcast_cell_ref` implementation to avoid running
        into a compile-error because of this very non-covariance!

        <details><summary>Click to see</summary>

        ```rust ,ignore
        impl<'u> dyn 'u + MyAny {
            fn downcast_cell_ref<'r, T : 'static>(
                self: &'r (dyn 'u + MyAny),
            ) -> Option<&'r Cell<&'u T>>
            {
                self.is::<Cell<&'static T>>().then(unsafe {
                    &*(self as *const Self as *const Cell<&T>)
                }) //                                     👆
            }
        }
        ```

        </details>

        <style>
        p > code.hljs {
            white-space: nowrap;
        }
        </style>

        Indeed, notice how, in the pointer casts, rather than `Cell<&'static i32>`, `Cell<&i32>` has
        been used instead, so that it can _directly_ be inferred to be a `Cell<&'u i32>`.

        Should that not had been done, and `Cell<&'static i32>` had been used in that pointer cast,
        then we would have been unable to get back a `Cell<&'u i32>` out of it due to the lack of
        covariance.

    ___

This means that now we need `'new_u` to be _exactly_ `'u`: if it ends up being _smaller_ than `'u`,
we'll be able to implement the `fn unsound` above using `we_do_a_lil_unsafe_2()`.

In other words, we need to make sure the returned `dyn 'u + …` is also, somehow, "lower-bounded" by
`'u` too: `'u` must not be able to shrink.

And this is a problem, since `dyn 'u + MyAny` is a type covariant in `'u`: `'u` is very much allowed
to shrink!

```rust ,edition2018
trait MyAny /* … */ { /* … */ }

fn shrink</* from */ 'big, /* to */ 'small>(
    b: Box<dyn 'big + MyAny>,
) -> Box<dyn 'small + MyAny>
where
    'big : 'small,
{
    b
}
# fn main() { println!("✅"); }
```

  - You may wonder _why_ that is. The reason for it is that `+ 'duration` expresses the
    `+ UsableWithin<'duration>` property, and if something is `UsableWithin<'some_duration>`,
    then _a fortiori_ it is `UsableWithin<'a_smaller_duration>`; much like when something is
    `Copy` then _a fortiori_ it is `Clone`.

How do we solve this?

Very easily, actually: just add an artificial `<'u>` generic lifetime parameter to our `MyAny`
trait.

Indeed, `Trait<'some_lt>`, does not have a specific meaning like `Trait + 'usability` does (which is
what allowed Rust to be lenient and allow shrinking that `+ 'usability`). Within an arbitrary/opaque
`Trait<'lt>` definition, that `'lt` may play any role, including APIs wherein shrinking `'lt` would
be unsound (like our own very case, obviously, but not only that). This means that Rust is not
allowed to modify that `<'lt>` in any way: it will keep it exactly as it initially appears (we say
that `dyn Trait<'lt>` is _invariant_ in `'lt`).

  - Alas, despite the `<'lt>` parameter on `Trait`, the `+ 'usability` parameter is still needed for
    the type, and `Box<dyn Trait<'lt>>` is sugar not for `Box<dyn Trait<'lt> + 'lt>` but for
    `Box<dyn Trait<'lt> + 'static>`, which is a type that combines the worst of both worlds:
      - the presence of `<'lt>` in the type makes it unable to be used beyond it ("`'lt`-infected");
      - the presence of (the implicit) `+ 'static` makes coërcing a concrete type to it more
        difficult (_e.g._, `&'lt i32 : 'static` does not hold).

    In other words, you'd have to prove that your concrete type is `: 'static` only for that
    property to be immediately thrown out of the window because of `<'lt>`.

    Conclusion: we'll have to "stutter" and talk about `dyn 'lt + Trait<'lt>`.

```rust, edition2018
use ::core::{any::TypeId, cell::Cell};

pub
trait MyAny<'lt> : seal::StaticSealed {
    fn type_id(&self) -> TypeId;
}

impl<T : ?Sized + 'static> MyAny<'_> for T {
    fn type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }
}

// Main trick: this, much like `Any`'s own `: 'static`, makes `: 'static` a
// mandatory step to be `MyAny`, but the big difference is that despite
// the requirement, we don't get the reverse implication: as far as Rust is
// concerned, there could exist `T : MyAny` for which `T : 'static` would
// not hold!
mod seal {
    pub trait StaticSealed {}
    impl<T : ?Sized + 'static> StaticSealed for T {}
}

// --------------------------------

// from there, the usual downcasting shenanigans:
impl<'u> dyn 'u + MyAny<'u> {
    pub
    fn is<T : 'static>(&self) -> bool {
        self.type_id() == TypeId::of::<T>()
    }

    pub
    fn downcast_bounded_ref<'r, T : 'static>(
        self: &'r (dyn 'u + MyAny<'u>),
    ) -> Option<&'r &'u T>
    {
        self.is::<&'static T>().then(|| unsafe {
            &*(self as *const Self as *const &'static T) // : &'r &'static T
                                                         // : &'r &'u      T
        })
    }

    fn downcast_cell_ref<'r, T : 'static>(
        self: &'r (dyn 'u + MyAny<'u>),
    ) -> Option<&'r Cell<&'u T>>
    {
        self.is::<Cell<&'static T>>().then(|| unsafe {
            &*(self as *const Self as *const Cell<&T>)
        })
    }
}

/// This is sound! 🥳
pub
fn we_do_a_lil_unsafe_2<'u>(
    r: Cell<&'u i32>,
) -> Box<dyn 'u + MyAny<'u>>
{
    // SAFETY:
    //  1. this `'static` is immediately erased to `dyn 'u + …`,
    //     so the resulting entity won't be usable beyond `'u`;
    //  2. the only way to extract this `Cell<&i32>` back from a `dyn MyAny` is
    //     through `downcast_cell_ref`, which also yields a `'u`-bounded Cell<&i32>
    //  3. Variance (or rather lack thereof): the input `r` is invariant in `'u`,
    //     and the returned `dyn 'u + MyAny<'u>` also is, so it won't be possible
    //     to shrink it.
    let r: Cell<&'static i32> = unsafe { ::core::mem::transmute(r) };
    Box::new(r)
}
```

<img
    src="https://user-images.githubusercontent.com/9920355/244114830-19584c98-aabe-42d4-9406-e9649268901b.png"
    height="200px"
/>

### Fully generalizing this pattern 🤯

So, now that we have managed to handle a few non-`'static` cases, it is time to try to generalize
these `we_do_a_lil_unsafe` operations: let's make it generic over some trait expressing the
following "split a `'u`-infected type into `'u`, and a `'static`/non-`'u`-infected type.

```rust ,ignore
//! In pseudo-code parlance:
&'u i32 = Combine<'u, &'static i32>
Cell<&'u i32> = Combine<'u, Cell<&'static i32>>
// let's be able to add extra such associations with manual `impl`s
```

The idea being that:
  - the latter can be `dyn`-erased,
  - and we'll strive to make it so the former (`'u`) be kept around in the type system at all times.

#### Implementation

 1. Express the `&'u i32 = Combine<'u, &'static i32>` intuition:

    ```rust ,ignore
    {{#include naive_any_example.rs:with-and-without-lifetime}}
    ```

    <details><summary>Bonus: the <code>Static</code> coherence wrapper</summary>

    ```rust ,ignore
    {{#include naive_any_example.rs:static}}
    ```

    ___

    </details>

 1. Now to tweak our `MyAny<'lt>` so as to no longer require `: 'static`
    while still having access to `TypeId`s

    ```rust ,ignore
    {{#include naive_any_example.rs:any}}
    ```

 1. From there, our `lil_unsafe` fns can be written _generically_, easily, and with no `unsafe`!

    ```rust ,ignore
    {{#include naive_any_example.rs:coerce}}
    ```

 1. Demo:

    ```rust ,ignore
    {{#include naive_any_example.rs:main}}
    ```

**[Full snippet playground](
https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=a8263b0938f0d0c8913be7c16bd56490)**
<details><summary>Click here to play with the full snippet inline</summary>

```rust ,edition2018,editable
{{#include naive_any_example.rs:all}}
```

___

</details>
