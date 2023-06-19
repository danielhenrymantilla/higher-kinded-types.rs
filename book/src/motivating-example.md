# A motivating example

### Situation

Imagine having:

```rust ,edition2018
struct Client {
    tier: u8,
    id: String,
}
```

and then wanting to _sort_ some `&mut [Client]` slice of `Client`s. The thing is, you don't want to
be implementing `Ord` for `Client`, since there is no canonical/absolute ordering of `Client`s:
  - sometimes you may want to sort them based on their `.id`;
  - and sometimes you may want to sort them based on their `.tier`.

So you cannot directly use `slice::sort()`, but luckily you notice there is a special API for
sorting based on a field of our choosing:

```rust ,edition2018
# struct Client { id: String, tier: u8 }
fn sort_clients_by_tier(cs: &mut [Client]) {
    cs.sort_by_key(|c| -> u8 { c.tier })
}
# fn main() { println!("✅"); }
```

So far, so good, but now say you want to implement the other sorting, the one based on the `.id`:

```rust ,compile_fail
# struct Client { id: String, tier: u8 }
fn sort_clients_by_id(cs: &mut [Client]) {
    cs.sort_by_key(|c| -> &String { &c.id })
}
# fn main() { println!("✅"); }
```

This fails! With:

```rust ,ignore
error: lifetime may not live long enough
 --> src/lib.rs:3:33
  |
3 | cs.sort_by_key(|c| -> &String { &c.id })
  |                 -     -         ^^^^^ returning this value requires that `'1` outlive `'2`
  |                 |     |
  |                 |     let's call the lifetime of this reference `'2`
  |                 has type `&'1 Client`
```

What happened?

The `.tier` case worked because `u8` was `Copy` so we directly returned it from our closure, but
for `.id`s we have to deal with `String`s, which are not `Copy` nor cheap to `Clone`, so the
following, even if it works, would be silly and is out of the question:

```rust ,edition2018
# struct Client { id: String, tier: u8 }
fn sort_clients_by_id(clients: &mut [Client]) {
    clients.sort_by_key(|client| client.id.clone())
}
```

This is basically the **6-year-old** issue of `slice::sort_by_key`.

> [`slice::sort_by_key` has more restrictions than `slice::sort_by`](https://github.com/rust-lang/rust/issues/34162)

The problem stems from _quantification_. Here is the signature of `sort_by_key`:

```rust ,ignore
fn sort_by_key<K>(
    self: &mut [Item],
    key_getter: impl FnMut(&Item) -> K,
)
where
    K : Ord,
```

 - (I've just replaced a named `<F>` with an anonymous inlined `impl FnMut` since I find it more
    readable that way).

The key observation is to focus on the hidden / implicitly-elided lifetime lurking behind the `&` in
that `&T` argument of the `key_getter` closure:

```rust ,ignore
//! pseudo-code!
fn sort_by_key<K>(
    self: &mut [T],
    key_getter: impl FnMut<'item>(&'item T) -> K,
    //                    ^^^^^^^  ^^^^^
)
where
    K : Ord,
```

  - Note: the above is pseudo-code, although I find it more readable that way for those not used to
    the `for<>` syntax. But for the sake of completeness, here is the real code:

    <details><summary>Click here to see</summary>

    ```rust ,ignore
    //! real code
    fn sort_by_key<K>(
        self: &mut [T],
        key_getter: impl for<'item> FnMut(&'item T) -> K,
        //               ^^^^^^^^^^        ^^^^^
    )
    where
        K : Ord,
    ```

    </details>

Mainly, notice how the signature is **not** the following one:

```rust ,ignore
//                     vvvvv
fn sort_by_key_simpler<'item, K>(
    self: &mut [T],
    key_getter: impl FnMut(&'item T) -> K,
                      // 👆
)
where
    K : Ord,
```

Now, if this is a detail you've never paid too much attention to, you should really stop, take a
good moment to stare at this change, trying to guess what the difference is: how is this differently
quantified?

___

The gist of it is the difference between "there exists x", spelled as `∃x` in mathematical shorthand
notation, and "for any x" (also called "for all x", "for each x", "for every x"), spelled as `∀x`.

  - Let's illustrate the difference with some basic examples:

    ```rust ,ignore
    // there exists a(t least one) number x, such that x ≥ 0.
    ∃x: i32, x >= 0
    ```

    which is `true`: `x = 1`, or `x = 0`, and basically any choice of a non-negative number will
    satisfy this predicate.

    ```rust ,ignore
    // for every number x, x ≥ 0
    ∀x: i32, x >= 0
    ```

    which is `false`: there exists a(t least one) number `x` such that `x ≥ 0` not hold,
    _i.e._, such that `x < 0`, _e.g._, `x = -1 `:

    ```rust ,ignore
    x = -1 ⇒ x < 0
    ⇒
    ∃x: i32, x < 0
    ⇔
    ∃x: i32, ¬(x >= 0)
    ⇔
    ¬(∀x: i32, x >= 0)
    ```

      - (notice how the negation of a ∀-quantified property involves exhibiting a ∃-quantified
        counter-example (and _vice-versa_): if you want to disprove that all cars are red, it
        suffices to find one non-red car; but if you wish to disprove that there exists a swan which
        is black you need to prove/observe that every swan is not black).


    But even if it may seem harder to find properties that hold for _every_ `x`, they actually
    exist!

    A basic example: `∀x: u32, x >= 0`.

    A more interesting one: `∀x: i32, x.saturating_mul(x) >= 0` (the famous `∀𝓍∈ℝ, 𝓍²≥0`)

While these differences can be a bit mind-bending at first, there is a point of view which I find
very handy, which is the _adversarial_ model. The idea is that you have, in front of you, a very
skilled adversary, and when you say something like `∀x: i32, x.saturating_mul(x) >= 0`, that is,
a "for any"-quantified property, what you are saying is basically kind of a "bet":

> I challenge you to find some `x: i32` so that `x.saturating_mul(x) >= 0` not hold. I bet you
can't!

Whereas something like `∃x: i32, x >= 0`, would rather be:

> Here is some `x` (_e.g._, `x=3`), as you can see it holds the required property.

So, to summarize:

  - `∀x` / "for any `x`": the adversary picks `x`, and it has to work no matter their choice!
  - `∃x` / "there exists `x`": _you_ pick `x`, and your adversary better make do with it no matter
    _your_ choice!

___

Back to the context of function signatures:

```rust ,ignore
//! pseudo-code.
fn sort_by_key<K>(
    self: &mut [T],
    key_getter: impl FnMut<'item>(&'item T) -> K,
)
where
    K : Ord,
```

**The outermost generic parameters** (such as `K` or the `F = impl FnMut…` parameters), **are picked
by the caller**. So they are `∃`-quantified for them, while _the callee_ has to treat the choices of
`K` and `F`, done by the "adversary" (the caller), as universal/`∀`-quantified "the caller may pick
_any_ `K` or `F` they want!".

  - A funny way to observe this is to go on a forum/discord and ask if the type of `f` in
    `fn example(f: impl FnMut())` is existential or universal. This kind of question is incomplete
    (it depends on whether you are talking from the point of view of the caller, or of the callee),
    and as is usually the case for such ill-asked mathematical questions, you'll find that people
    may rush to answer _their_ (implicit, and perhaps unconscious) choice of point of view, arguing
    with the others 😄

So `K, F` are picked by the caller (they do have to uphold that their specific single choice of
`F` and `K` do uphold the `F: FnMut…` and `K: Ord` constraints; from the point of view of the callee,
the `F` and `K` picked could be _any choice_, but _at least_ they know that `F: FnMut…` and `K: Ord`
will hold).

What about `'item`? Well, that's the key difference between `sort_by_key_simpler()` and
`sort_by_key()`.

  - The former, `fn sort_by_key_simpler()`, has `'item` amongst its _outermost_ generic parameters,
    so it is also
    picked by the caller (in this silly example of mine, where there is nothing else constraining the
    `'item` choice, the caller could even go and pick `'item = 'static`!).

    ```rust ,ignore
    //                     vvvvv
    fn sort_by_key_simpler<'item, K>(
        self: &mut [T],
        key_getter: impl FnMut(&'item T) -> K,
    )
    where
        K : Ord,
    ```

    This means that if the caller picks some `'item` (such as `'item = 'static`), they can _then_
    pick a `K` so that it includes this lifetime, _e.g._, `K = &'item String`, and _then_ an

    ```rust ,ignore
    F = impl FnMut(&'item Client) -> &'item String
    ```

    and all is good.

  - The latter, `fn sort_by_key`, on the other hand, **does not have `'item` amongst its _outermost_
    generic parameters**. It has `<'item>` amongst the generic parameters of the `impl FnMut<'item>`
    itself (_inner_ generics) in pseudo-code, which is what the real `impl for<'item> FnMut(…)`
    syntax means anyways.

    ```rust ,ignore
    //! pseudo-code (real syntax: `for<'item> FnMut(&'item T) -> K`)
    fn sort_by_key<K>(
        self: &mut [T],
        cb: impl FnMut<'item>(&'item T) -> K,
        //                     ^^^^^   ^^^^^
    )
    where
        K : Ord,
    ```

    For a `fn cb()` definition to meet such a signature, it would have to be defined as:

    ```rust ,ignore
    fn cb<'item>(item: &'item Client) -> &'item String // for instance
    ```

    and here we have an "outermost" generic parameter like we are used to. Which means it is picked
    by the caller. But who is the _caller_, here? We're talking of _the caller of `cb`_! Who is
    calling `cb` within `sort_by_key()`? The _body_ of `sort_by_key()`, that is, the _callee_!

    In other words, the **`<'item>` _inner_ generic parameter is picked by the _callee_**, not the
    caller!

      - From the point of view of the caller, it is thus a _universal_ lifetime parameter,
        _i.e._, a "for all"-quantified one, hence the `for<'item>` syntax:

        ```rust ,ignore
        for/*any*/<'item> FnMut(&'item T) -> K
        ```

    Now let's imagine a caller calling into `sort_by_key()`, and wanting the closure to return a
    `&String`:

    The outermost generic parameters are `K` and `F = impl FnMut…`. So they can pick some
    lifetime which we will call `'k`, and then `K = &'k String`, and then:
    ```rust ,ignore
    F = impl FnMut<'item>(&'item Client) -> &'k String
    ```

    See the problem? The return of our closure is `&'k String` for _some_ `'k` picked by the caller.
    But what the closure will receive, from the callee / the body of `sort_by_key`, is some
    `&Client` **with some callee-chosen `'item` lifetime**, _which may very well be smaller than
    `'k`_!

      - To insist on this point, `'k` cannot be `'item`, because the former is picked by the caller,
        whilst the latter is picked by the callee, "after" the caller, or independently from the
        caller. So these are independent/distinctly-named lifetimes (and as we will see below,
        it won't be possible for `'item : 'k` to hold, let alone `'k = 'item`).

      - To illustrate this, let's share again that error message above, but renaming `'1` as
        `'item`, and `'2` as `'k`:

        ```rust ,ignore
        error: lifetime may not live long enough
         --> src/lib.rs:3:33
          |
        3 | cs.sort_by_key(|c| -> &String { &c.id })
          |                 -     -         ^^^^^ returning this value requires
          |                 |     |               that `'item` outlive `'k`
          |                 |     |
          |                 |     let's call the lifetime of this reference `'k`
          |                 has type `&'item Client`
          // added by me:              ^^^^^
          //            :          for some universal/higher-order/callee-chosen
          //            :          `'item` lifetime.
        ```

          - In mathematical parlance, type-checking the lifetimes of the closure requires the
            following property to hold:

            ```rust ,ignore
            /* From the point of the caller (we are type-checking the call-site!): */

            //  caller-chosen
            //  v                  ≥
                ∃'k, ∀'item, 'item : 'k
            //       ^             ^ from borrow-checking `&c.id`
            //     callee-chosen
            ```

            which is `false`, hence the compilation error.

            ___

            Another point of view, related to this, is that **caller-chosen lifetimes cannot be
            smaller than the scope of the `fn` body of the callee**:

            ```rust , ignore
            fn sort_by_key…(…) … where … { // ---+----------------+
                /* … body … */                // | 'fn            | 'k
            } // <- end of fn body --------------+                |
                                                               // … (wiggle-room for the caller)
            // <--------------------------------------------------+
            //   the shortest possible `'k` that a caller can pick must end *after*
            //   the closing `}`.
            ```

            <details><summary>Click here to see how to use this to prove that <code>false</code> claim above</summary>

            If we call `'fn`, the lifetime/scope of code spanning exactly until that
            closing `}` at the end of the body of `fn sort_by_key()`, we then have:

            ```rust ,ignore
            /* point of view of the callee */

            ∀'k, 'k > 'fn
            ⇔
            ∀'k, 'k ≥ 'fn and 'fn ≠ 'k
            ⇔
            ∀'k, 'k ≥ 'fn and ¬('fn ≥ 'k)
            ⇔
            ∀'k, 'k : 'fn and ¬('fn : 'k)
            ```

            So, from there, we can drop the `'k : 'fn` part, so as to have:

            ```rust ,ignore
            ∀'k, ¬('fn : 'k) // for every `'k`, `'fn : 'k` does not hold.
            ⇒
            ∀'k, ∃'item = 'fn, ¬('item : 'k) // for every `'k`, there exists an `'item` (such as `'fn`) so that `'item : 'k` not hold
            ⇒
            ∀'k, ∃'item, ¬('item : 'k) // for every `'k`, there exists an `'item` so that `'item : 'k` not hold
            ⇔
            ¬(∃'k, ∀'item, 'item : 'k) // NOT(there exists a 'k, so that for every `'item`, 'item : 'k` hold)
            ```

            QED

              - (in practice the real `'item` picked by the callee is even, itself, shorter than
                `'fn`, but Rust does not even need to think about it, since when `'item` is as big
                `'fn`, it's already not big enough for that `'k`. Let alone / _a fortiori_ for
                smaller `'item` lifetimes).

            ___

            </details>


    So the only way to return a `&'k String` from such a closure would be by not needing
    `'item : 'k` to hold, _i.e._, by not borrowing that `String` from the client, _i.e._, by
    returning an _unrelated_ `&'k String` value, _i.e._, something _owned separately_.

    Hence the restriction of `sort_by_key`: it can only be used to return (separately borrowed or)
    owned types!

### Solving the returned borrow problem

So, the problem here was that the return type `-> …` in the `impl FnMut<'item>(&'item T) -> …`
closure signature was not _naming_ `'item`, and thence, unable to capture it / depend on it, so
that borrowing the input `&'item T` was not possible.

So what about naming `'item` in the return type?

```rust ,edition2018
use ::core::cmp::Ord;
#
# struct Client { id: String, tier: u8 }

fn sort_by_key_ref<K : ?Sized> (
    clients: &'_ mut [Client],
mut get_key: impl for<'item> FnMut(&'item Client) -> &'item K,
)
where
    // requirement for the caller: no matter the choice of `'item`
    // by your "adversary" / the callee, `&'item K : Ord` needs to hold.
    // i.e., `∀'item, &'item K : Ord`:
    for<'item>
        &'item K : Ord
    ,
{
    clients.sort_by(|a: &Client, b: &Client| {
        let ka = get_key(a);
        let kb = get_key(b);
        Ord::cmp(&ka, &kb)
    })
}

fn sort_by_id(clients: &mut [Client]) {
    sort_by_key_ref(clients, |client| &client.id); // OK 🥳
}
#
# fn main() { println!("✅"); }
```

This does work 🥳

But what about the other example, sorting by `tier`?

```rust ,edition2018
# use ::core::cmp::Ord;
#
# struct Client { id: String, tier: u8 }
#
# fn sort_by_key_ref<K : ?Sized> (
#     clients: &'_ mut [Client],
#     key_getter: impl for<'item> FnMut(&'item Client) -> &'item K,
# )
# where
#     // requirement for the caller: no matter the choice of `'item`
#     // by your "adversary" / the callee, `&'item K` needs to hold.
#     // i.e., `∀'item, &'item K : Ord`:
#     for<'item>
#         &'item K : Ord
#     ,
# {
#     let mut get_key = key_getter;
#     clients.sort_by(|a: &Client, b: &Client| {
#         let ka = get_key(a);
#         let kb = get_key(b);
#         Ord::cmp(&ka, &kb)
#     })
# }
#
fn sort_by_tier(clients: &mut [Client]) {
    sort_by_key_ref(clients, |client| client.tier); // Error!
}
#
# fn main() { println!("✅"); }
```

This fails with:

```rust ,ignore
error[E0308]: mismatched types
  --> src/main.rs:26:39
   |
26 |     sort_by_key_ref(clients, |client| client.tier); // Error!
   |                                       ^^^^^^^^^^^
   |                                       |
   |                                       expected `&_`, found `u8`
   |                                       help: consider borrowing here: `&client.tier`
   |
   = note: expected reference `&_`
                   found type `u8`
```

since, indeed, our `-> u8` return type is no longer able to match our **less
generic** `-> &'item _` signature constraint.

In this case, the solution is easy: even though `u8` is `Copy`, if the API
wants a borrow to be returned, we won't sweat it, and comply:

```rust ,ignore
clients.sort_by_key_ref(|client| &client.tier);
//                               👆
```

which does compile.

### Can we be _fully generic_ over the return type?

But this is not fully satisfactory. Say, now, that we want to sort by `.tier`, but _in reverse
order_ (the `.sort_…` functionality always sorts in ascending order w.r.t. what `Ord` says).

It turns out that there is a very handy adapter which swaps what `Ord` says, precisely to get
_descending order_ for the wrapped values: [`Reverse`].

[`Reverse`]: https://doc.rust-lang.org/stable/std/cmp/struct.Reverse.html

```rust ,ignore
use ::core::cmp::Reverse;

sort_by_key_ref(clients, |client| Reverse(&client.tier));
```

```rust ,ignore
error[E0308]: mismatched types
  --> src/main.rs:28:39
   |
28 | sort_by_key_ref(clients, |client| Reverse(&client.tier));
   |                                   ^^^^^^^^^^^^^^^^^^^^^
   |                                   |
   |                                   expected `&_`, found `Reverse<&u8>`
   |                                   help: consider borrowing here: `&Reverse(&client.tier)`
   |
   = note: expected reference `&_`
                 found struct `Reverse<&u8>`
```

Ah, right, we need to return a borrow, so let's do what the diagnostic suggests:

```rust ,ignore
use ::core::cmp::Reverse;

sort_by_key_ref(clients, |client| &Reverse(&client.tier));
//                                👆
```

```rust ,ignore
error[E0515]: cannot return reference to temporary value
  --> src/main.rs:28:39
   |
28 | sort_by_key_ref(clients, |client| &Reverse(&client.tier));
   |                                   ^--------------------
   |                                   ||
   |                                   |temporary value created here
   |                                   returns a reference to data owned by the current function
```

Hmmm

<img
  src="https://user-images.githubusercontent.com/9920355/246637508-5a96ab96-fa43-4f25-8f08-8993b2bd6bab.png"
  alt="No, no. It's got a point"
  title="No, no. It's got a point"
  height="150px"
/>

Yeah, true: wrapping value in `Reverse` creates a new _owned_ value (inside our `Reverse<T>`), which
is created and owned by the closure's internal `fn` scope, so we cannot return a borrow to it!

Granted, for this case we could just use the owned-return-type API (_i.e._, the stdlib's
`sort_by_key`):

```rust ,edition2018
use ::core::cmp::Reverse;
# struct Client { id: String, tier: u8 } let clients: &mut [Client] = &mut [];

<[_]>::sort_by_key(clients, |client| Reverse(client.tier));
                                 // ^ no `&` ^
# println!("✅");
```

But by now it should be clear how we are running into **clear API limitations**. The return type of:

  - `sort_by_key` is fully generic, but cannot borrow from the input `client` / cannot name `'item`;

  - `sort_by_key_ref` is able to borrow from the input `client`, but by hardcoding a `&'item …`
    "shape" for such a borrowing (with the `…` part still not being allowed to borrow).

To illustrate how insactisfactory this is, now consider wanting to sort based on the `.id`, again,
but this time, [`Reverse`]d!

  - we cannot use `sort_by_key_ref` with <code><span style="color: red;">\&</span>Reverse(…)</code>
    as we've already seen with `.tier` (the `Reverse` object is an owned "temporary" of the
    closure);

  - we cannot use `sort_by_key` either (unless we were to `.clone()` the `String` ❌), since even
    if we tried returning <code>Reverse(<span style="color: red;">\&</span>client.id)</code>, then
    that inner `&` would still be borrowing from `client`, so we don't have an "owned/independent"
    return type.

    ```rust ,ignore
    use ::core::cmp::Reverse;

    //                 +---------- nope nope nope -------------+
    //                /                                         \
    //             vvv                                           vv
    <[_]>::sort_by_key(clients, |client: &'_ Client| -> Reverse<&'_ String> {
        Reverse(&client.tier)
    });
    ```

So I guess we'd need yet another dedicated API just for reversing?

```rust ,edition2018
use ::core::cmp::{Ord, Reverse};
#
# struct Client { id: String, tier: u8 }

fn sort_by_key_ref_rev<K : ?Sized> (
    clients: &'_ mut [Client],
mut get_key: impl for<'item> FnMut(&'item Client) -> Reverse<&'item K>,
                                                  // ++++++++        +
)
where
    for<'item> &'item K : Ord,
{
    clients.sort_by(|a: &Client, b: &Client| {
        let ka = get_key(a);
        let kb = get_key(b);
        Ord::cmp(&ka, &kb)
    })
}

fn sort_by_tier_reversed(clients: &mut [Client]) {
    sort_by_key_ref_rev(clients, |client| Reverse(&client.tier));
}

fn sort_by_id_reversed(clients: &mut [Client]) {
    sort_by_key_ref_rev(clients, |client| Reverse(&client.id));
}
#
# fn main() { println!("✅"); }
```

So now we got:

  - `<[_]>::sort_by_key()` for the `-> _` case;
  - `sort_by_key_ref()` for the `-> &'_ _` case;
  - `sort_by_key_ref_rev()` for the `-> Reverse<&'_ _>` case.

Needless to say this is not scaling well; at all. In fact, we are ending up with the very opposite
of what generic APIs should be featuring!

For instance, now consider wanting to sort based on the `.tier`, and _then_ sorting (within each
equal/tied `.tier` group), based on the `.id`. In Rust we have, similar to [`Reverse`], a handy
generic type to achieve this "sort based on `T`, and then break each `T` tie based on `U`":
`(T, U)`!

  - Yes, something as simple as `(T, U)` is exactly the tool for this. This kind of ordering has a
    name, called _lexicographic_ order, which stems from how we sort words when we do it
    _alphabetically_, like dictionaries do: `"axis"` comes before `"elephant"` even though `x` comes
    before `l` because we've, first, sorted based on `a` _vs._ `e`, and since there was no tie,
    there is nothing else to do (_vs._ sorting against `"acorn"`, wherein there will be a "tie" with
    `a`, and that's when we will compare each second character, `x` _vs._ `c`, to sort them).

    See, then, how the official Rust documentation guarantees this:

      - [highlight link](https://doc.rust-lang.org/1.70.0/std/primitive.tuple.html#impl-PartialOrd%3C(T,)%3E-for-(T,):~:text=The%20sequential%20nature%20of%20the%20tuple%20applies%20to%20its%20implementations%20of%20various%20traits.%20For%20example%2C%20in%20PartialOrd%20and%20Ord%2C%20the%20elements%20are%20compared%20sequentially%20until%20the%20first%20non%2Dequal%20set%20is%20found.)

      - [raw link (if the above does not work)](https://doc.rust-lang.org/1.70.0/src/std/primitive_docs.rs.html#987-989)

```rust ,edition2018,compile_fail
# use std::cmp::Reverse;
#
# struct Client { id: String, tier: u8 }
#
# fn sort_by_key_ref<K : ?Sized> (
#     clients: &'_ mut [Client],
#   mut get_key: impl for<'item> FnMut(&'item Client) -> &'item K,
# )
# where
#     for<'item>
#         &'item K : Ord
#     ,
# {
#     clients.sort_by(|a: &Client, b: &Client| {
#         let ka = get_key(a);
#         let kb = get_key(b);
#         Ord::cmp(&ka, &kb)
#     })
# }
#
# let clients = &mut [];
sort_by_key_ref(clients, |c: &Client| -> (&u8, &String) { // Error, expected `-> &_`
  (&c.tier, &c.id)
});

sort_by_key_ref(clients, |c: &Client| -> (&u8, Reverse<&String>) { // Error, expected `-> &_`
  (&c.tier, Reverse(&c.id))
});
```

> Error: expected `[closure@main.rs:24:26]` to be a closure that returns `&_`, but it returns `(&u8, …)`

What should we do? Keep duplicating the API for each specific combination we may think of? Surely
not!

  - Granted, for this specific situation, the `slice.sort_by(|a, b| -> Ordering { … })` API is an
    incredibly convenient way to sidestep all of these issues (since the return type will always be
    a non-borrowing `Ordering`, while being able to work with borrows inside the closure's body).

    In fact, for the `(&c.tier, &c.id)` case, it's probably even _more readable_ using that than
    relying on tuple's lexicographic semantics:

    ```rust ,edition2018
    use ::core::cmp::{Ord, Ordering}; // `sh` users be like: `cmp::Ord{,ering}` 🙃

    # struct Client { id: String, tier: u8 }
    #
    # let clients: &mut [Client] = &mut [];
    #
    clients.sort_by(|a, b| {
        Ordering::Equal
            .then_with(|| Ord::cmp(&a.tier, &b.tier))
            .then_with(|| Ord::cmp(&a.id, &b.id).reverse())
    });
    # println!("✅");
    ```

    So, realistically, nowadays, in the current non-HKT-educated culture, and especially at $work,
    or when working with many other people, it's probably best to be doing this with `.sort_by()`.

    But here is a peek preview of what we'll be able to work with, later on, and compare the
    verbiage above with the elegant brevity of:

    ```rust ,ignore
    clients.sort_by_dependent_key::<Gat![(Tier, Reverse<&Id>)]>(|c| (
        c.tier,
        Reverse(&c.id),
    ));
    ```

    and, in future Rust, we can even envision:

    ```rust ,ignore
    clients.sort_by_dependent_key(for<'a> |c: &'c Client| -> (Tier, Reverse<&'c Id>) {(
        c.tier,
        Reverse(&c.id),
    )});
    ```

    [Just Working™](https://rust.godbolt.org/z/3ncevrK8W)

Let's summarize

#### A recap of the different return types seen so far:

```rust ,ignore
// Given:
type Tier = u8; // Copy
type Id = String; // "expensive" to clone.
struct Client { tier: Tier, id: Id }

// We've seen the following return types so far:
type A<'item> = Tier; // sort_by_key ✅
type B<'item> = &'item Id; // sort_by_key_ref ✅
type C<'item> = &'item Tier; // sort_by_key_ref ✅
type D<'item> = Reverse(Tier); // sort_by_key ✅
type E<'item> = Reverse(&'item Id); // ❌
type F<'item> = (Tier, &'item Id); // ❌
type G<'item> = (Tier, Reverse<&'item Id>); // ❌
```

Notice how they've all been written so as to fit the shape of:

```rust ,ignore
type Output<'item> = …;
```

since really, this is the abstraction we've been intuitively thinking about, right? I've been
ranting about the borrowing `-> &'item _` return type still not being "generic enough": it being
_overly restrictive_, for a while, now 😄

Really, what we've been wanting to say is:

  - there exists some `type Output<'item> = …` definition, so that, for any choice of `'item` made
    by the callee, we have the return type of our closure on a `&'item Client` being that `…`:

    ```rust ,ignore
    //! pseudo-code!

             // there exists
             // v
    fn intuition<Output /* <'_> */>(
        // so that, for any choice of `'item`, it match the return type of our closure
        get_key: impl for<'item> FnMut(&'item Client) -> Output<'item>,
    )
    ```

For instance, given `type E<'item> = Reverse<&'item String>;`, we'd like to call:

```rust ,ignore
//! pseudo-code!
use ::core::cmp::Reverse;

type E<'item> = Reverse<&'item String>;

intuition::<E>(|c: &'_ Client| -> E<'_> {
    Reverse(&c.id)
})
```

Let's see what real Rust code has to say about it, then, shall we?

```rust ,edition2018,compile_fail
use ::core::cmp::Reverse;
# struct Client { id: String }

fn intuition<Output /* <'_> */>(
    /* let's not bother with `&mut [Client]` yet */
    _get_key: impl for<'item> FnMut(&'item Client) -> Output<'item>,
)
{}

type E<'item> = Reverse<&'item String>;

intuition::<E>(|c: &'_ Client| -> E<'_> {
    Reverse(&c.id)
});
# println!("✅");
```

This yields:

```rust ,ignore
error[E0109]: lifetime arguments are not allowed on type parameter `Output`
 --> src/main.rs:8:62
  |
8 |     _get_key: impl for<'item> FnMut(&'item Client) -> Output<'item>,
  |                                                       ------ ^^^^^ lifetime argument not allowed
  |                                                       |
  |                                                       not allowed on type parameter `Output`
  |
note: type parameter `Output` defined here
 --> src/main.rs:6:14
  |
6 | fn intuition<Output /* <'_> */>(
  |              ^^^^^^

For more information about this error, try `rustc --explain E0109`.
```
