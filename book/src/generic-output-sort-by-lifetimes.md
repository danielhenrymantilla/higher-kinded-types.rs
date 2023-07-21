# Can we be fully generic over the return type?

Say, now, that we want to sort by `.tier`, but _in reverse order_ (the `.sort_…` functionality
always sorts in ascending order w.r.t. what `Ord` says).

### Example: `Reverse`d ordering

It turns out that there is a very handy adapter which swaps what `Ord` says, precisely to get
_descending order_ for the wrapped values: [`Reverse`].

[`Reverse`]: https://doc.rust-lang.org/stable/std/cmp/struct.Reverse.html

```rust ,ignore
use ::core::cmp::Reverse;

sort_by_key_ref(clients, |client| Reverse(&client.tier));
```

Alas, this fails!

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

which causes:

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

Yeah, true: wrapping the value in `Reverse` creates a new _owned_ value (inside our `Reverse<T>`),
which is created and owned by the closure's internal `fn` scope, so we cannot return a borrow to it!

Granted, for this case we could just use the owned-return-type API (_i.e._, the stdlib's
`sort_by_key`):

```rust ,edition2018
use ::core::cmp::Reverse;
# struct Client { id: String, tier: u8 } let clients: &mut [Client] = &mut [];

<[_]>::sort_by_key(clients, |client| Reverse(client.tier));
                                 //  ^ no &  ^
# println!("✅");
```

But by now it should be clear that we are running into **clear API limitations**.
The return type of:

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
    //               v                                           vv
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
        let ka: Reverse<_> = get_key(a);
        let kb: Reverse<_> = get_key(b);
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

### Example: chained/lexicographic ordering

For instance, now consider wanting to sort based on the `.tier`, and _then_ sorting (within each
equal/tied `.tier` group), based on the `.id`. In Rust we have, similar to [`Reverse`], a handy
generic type to achieve this "sort based on `T`, and then break each `T` tie based on `U`". Have you
guessed it? It's our good ol' 2-tuple type: `(T, U)`!

  - Yes, something as simple as `(T, U)` is exactly the tool for this. This kind of ordering has a
    name, called _lexicographic_ order, which stems from how we sort words when we do it
    _alphabetically_, like dictionaries do: `"axis"` comes before `"elephant"` even though `x` comes
    _after_ `l` because we've, first, sorted based on `a` _vs._ `e`, and since there was no tie,
    there is nothing else to do (_vs._ sorting against `"acorn"`, wherein there will be a "tie" with
    `a`, and that's when we will compare each second character, `x` _vs._ `c`, to sort them, and so
    on).

    See, then, how the official Rust documentation guarantees this:

      - [highlight link](https://doc.rust-lang.org/1.70.0/std/primitive.tuple.html#impl-PartialOrd%3C(T,)%3E-for-(T,):~:text=The%20sequential%20nature%20of%20the%20tuple%20applies%20to%20its%20implementations%20of%20various%20traits.%20For%20example%2C%20in%20PartialOrd%20and%20Ord%2C%20the%20elements%20are%20compared%20sequentially%20until%20the%20first%20non%2Dequal%20set%20is%20found.)

      - [raw link (if the above does not work)](https://doc.rust-lang.org/1.70.0/src/std/primitive_docs.rs.html#987-989)

This, thus, ought to yield something along the lines of:

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
// or, with `Reverse` thrown into the mix:
sort_by_key_ref(clients, |c: &Client| -> (&u8, Reverse<&String>) { // Error, expected `-> &_`
    (&c.tier, Reverse(&c.id))
});
```

But we keep getting the infamous:

> Error: expected `[closure@main.rs:24:26]` to be a closure that returns `&_`, but it returns `(&u8, …)`

What should we do? **Keep _duplicating_ the API for each specific combination we may think of?**
Surely not!

  - Granted, for this specific situation, the `slice.sort_by(|a, b| -> Ordering { … })` API is an
    incredibly convenient way to sidestep all of these issues (since the return type will always be
    a non-borrowing `Ordering`, while being able to work with borrows inside the closure's body).

    In fact, for the `(&c.tier, &c.id)` case, it is probably even _more readable_ to be using that,
    rather than relying on tuple's lexicographic semantics:

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

    But here is a peek preview of what we'll be able to work with, [later on], and compare the
    verbiage above with the elegant brevity of:

    [later on]: hkts-sort-by-lifetimes.md#bonus-as-an-extension-method

    ```rust ,ignore
    clients.sort_by_dependent_key::<ForLt!{ (Tier, Reverse<&Id>) }>(|c| (
        c.tier,
        Reverse(&c.id),
    ));
    ```

    and, in future Rust, we can even envision:

    ```rust ,ignore
    clients.sort_by_dependent_key(for<'c> |c: &'c Client| -> (Tier, Reverse<&'c Id>) {(
        c.tier,
        Reverse(&c.id),
    )});
    ```

    [Just Working™](https://rust.godbolt.org/z/3ncevrK8W)

Let's summarize

### A recap of the different return types seen so far:

```rust ,ignore
// Given:
struct Client { tier: Tier, id: Id }
// with:
type Tier = u8; // Copy
type Id = String; // "expensive" to clone.

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

since really, this is the abstraction we've been intuitively thinking about, right? You'll agree
I've been ranting about "the borrowing `-> &'item _` return type not being 'generic enough' / it
being _overly restrictive_" for a good while, now 😆

Really, what we've been wanting to say is:

  - there exists some `type Output<'item> = …` definition, so that, for any choice of `'item` made
    by the callee, we have the return type of our closure on a `&'item Client` being that
    `Output<'item>`:

    ```rust ,ignore
    //! pseudo-code!

             // there exists
             // v
    fn intuition<Output /* <'_> */>(
        // so that, for any choice of `'item`, it shall match the return type of our closure
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
    _get_key: impl for<'item>
        FnMut(&'item Client) -> Output<'item>
    ,
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
 --> src/main.rs:9:40
  |
9 |         FnMut(&'item Client) -> Output<'item>
  |                                 ------ ^^^^^ lifetime argument not allowed
  |                                 |
  |                                 not allowed on type parameter `Output`
  |
note: type parameter `Output` defined here
 --> src/main.rs:6:14
  |
6 | fn intuition<Output /* <'_> */>(
  |              ^^^^^^

For more information about this error, try `rustc --explain E0109`.
```

Hmm, is that:

 1. a standalone `Output` generic type,
 1. to which we ought to be able to feed a `<'item>` lifetime parameter?

Right, it looks like it is time to unleash our `::higher-kinded-types` machinery on this problem!
