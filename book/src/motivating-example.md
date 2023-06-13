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

Now, if this is a detail you've never paid to much attention to, you should really stop, take a good
moment to stare at this difference, trying to guess what the difference is: how is this differently
quantified.

___

The gist of it is the difference between "there exists x", spelled as `∃x` in mathematical shorthand
notation, and "for any x" (also called "for all x", "for each x", "for every x"), spelled as `∀x`.

  - Some basic examples of this to illustrate the difference:

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

    which is `false`: there exists at least a(t least one) number `x` such that `x ≥ 0` not hold,
    _i.e._, such that `x < 0`, _e.g._, `x = -1 `:

    ```rust ,ignore
    x = -1
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

> Here is some `x` I can show you that holds the required property, to prove I was right!

So, to summarize:

  - `∀x` / "for any `x`": the adversary picks `x`, and it has to work no matter their choice!
  - `∃x` / "there exists `x`": _you_ pick `x`, and your adversary better make do with it no matter
    _your_ choice!

Back to the context of function signatures, **the outermost generic parameters** (such as `K` or the
`F = impl FnMut…` parameters), **are picked by the caller**. So they are `∃`-quantified for them,
while _the callee_ has to treat the choices of `K` and `F`, done by the "adversary" (the caller),
as universal/`∀`-quantified "the caller may pick _any_ `K` or `F` they want!".

  - A funny way to observe this is to go on a forum/discord and ask if the type of `f` in
    `fn example(f: impl FnMut())` is existential or universal. This kind of question is incomplete
    (it depends on whether you are talking from the point of view of the caller, or of the callee),
    and as is usually the case for such ill-asked mathematical questions, you'll find that people
    may rush to answer _their_ (implicit, and perhaps unconcious) choice of point of view, arguing
    with the others 😄

So `K, F` are picked by the caller (they do have to uphold that their specific single choice of
`F` and `K` do uphold that `F: FnMut…` and that `K: Ord`; from the point of view of the callee,
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
    pick `K` including it, _e.g._, `K = &'item String`, and _then_ an
    `F = impl FnMut(&'item Client) -> &'item String` and all is good.

  - The latter, `fn sort_by_key`, on the other hand, **does not have `'item` amongst its _outermost_
    generic parameters**. It has `<'item>` amongst the generic parameters of the `impl FnMut<'item>`
    itself (_inner_ generics) in pseudo-code, which is what the real `impl for<'item> FnMut(…)`
    syntax means anyways.

    ```rust ,ignore
    //! pseudo-code (real syntax: `for<'item> FnMut(&'item T) -> K`)
    fn sort_by_key<K>(
        self: &mut [T],
        key_getter: impl FnMut<'item>(&'item T) -> K,
        //                     ^^^^^   ^^^^^
    )
    where
        K : Ord,
    ```

    This means that for a `fn cb()` definition to meet such a signature, it would have to be defined
    as:

    ```rust ,ignore
    fn cb<'item>(item: &'item Client) -> &'item String // for instance
    ```

    and here we have an "outermost" generic parameters like we are used to. Which means it is picked
    by the caller… **of `cb`**! And who is calling `cb` within `sort_by_key()`? The _body_ of
    `sort_by_key()`, that is, the _callee_!

    In other words, the **`<'item>` _inner_ generic parameter is picked by the _callee_**, not the
    caller! (from the point of view of the caller, it is thus a _universal_ lifetime parameter,
    _i.e._, a "for all"-quantified one, hence the `for<'item>` syntax).

    Now let's imagine a caller calling into `sort_by_key()`, and wanting the closure to return a
    `&String`:

    The outermost generic parameters are `K` and `F = impl FnMut…`. So they can pick some
    lifetime which we will call `'k`, and then `K = &'k String`, and then:
    ```rust ,ignore
    F = impl FnMut<'item>(&'item Client) -> &'k String
    ```

    See the problem? The return of our closure is `&'k String` for _some_ `'k` picked by the caller.
    But what the closure will receiver, from the callee / the body of `sort_by_key`, is some
    `&Client` **with some callee-chosen `'item` lifetime**, _which may very well be smaller than
    `'k`!

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
          // added by me:              ^^^^^ for some universal/higher-order/callee-chosen
          //                                 `'item` lifetime.
        ```

          - In mathematical parlance, type-checking the lifetimes of the closure requires the
            following property to hold:

            ```rust ,ignore
            /* From the point of the caller (we are type-checking the call-site!): */

            //  caller-chosen
            //  v                  ≥
                ∃'k, ∀'item, 'item : 'k
            //       ^
            //       callee-chosen
            ```

            which is `false`, hence the compilation error.

            Another point of view related to this is that **caller-chosen lifetimes cannot be
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
use ::std::ord::Ord;
#
# struct Client { id: String, tier: u8 }

fn sort_by_key_ref<K : ?Sized> (
    self: &'_ mut [Client],
    key_getter: impl for<'item> FnMut(&'item Client) -> &'item K,
)
where
    // requirement for the caller: no matter the choice of `'item`
    // by your "adversary" / the callee, `&'item K` needs to hold.
    // i.e., `∀'item, &'item K : Ord`:
    for<'item>
        &'item K : Ord
    ,
{
    let mut get_key = key_getter;
    self.sort_by(|a: &Client, b: &Client| {
        let ka = get_key(a);
        let kb = get_key(b);
        Ord::cmp(&ka, &kb)
    })
}

fn sort_by_id(clients: &mut [Client]) {
    clients.sort_by_key_ref(|client| &client.id); // OK 🥳
}
#
# fn main() { println!("✅"); }
```

This does work 🥳

But what about the other example, sorting by `tier`?

```rust ,edition2018
# use ::std::ord::Ord;
#
# struct Client { id: String, tier: u8 }
#
# fn sort_by_key_ref<K : ?Sized> (
#     self: &'_ mut [Client],
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
#     self.sort_by(|a: &Client, b: &Client| {
#         let ka = get_key(a);
#         let kb = get_key(b);
#         Ord::cmp(&ka, &kb)
#     })
# }
#
fn sort_by_tier(clients: &mut [Client]) {
    clients.sort_by_key_ref(|client| client.tier); // Error!
}
#
# fn main() { println!("✅"); }
```

This fails with:

```rust ,ignore
ERROR MESSAGE HERE
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

But this is not fully satisfactory. Say, now, that we want to sort by `.tier`, but _in reverse
order_ (the `.sort_…` functionality always sorts in ascending order w.r.t. what `Ord` says).

It turns out that there is a very handy adapter which swaps what `Ord` says, precisely to get
_descending order_ for the wrapped values: `Reverse` TODO LINK.

```rust ,ignore
use ::core::ord::Reverse;

clients.sort_by_key_ref(|client| Reverse(client.tier));
```

Ah, right, we need to return a borrow:

```rust ,ignore
use ::core::ord::Reverse;

clients.sort_by_key_ref(|client| &Reverse(client.tier));
//                               👆
```





### Can we be _fully generic_ over the return type?
