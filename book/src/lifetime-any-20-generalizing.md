# Fully generalizing this pattern 🤯

So, now that we have managed to handle a few non-`'static` cases, it is time to try to generalize
these `we_do_a_lil_unsafe` operations: let's make it generic over some trait expressing the
following "split a `'u`-infected type into `'u`, and a `'static`/non-`'u`-infected type.

```rust ,ignore
//! In pseudo-code parlance:
type &'u i32 = Combine<'u, &'static i32>
type Cell<&'u i32> = Combine<'u, Cell<&'static i32>>
// let's be able to add extra such associations with manual `impl`s
```

The idea being that:
  - the latter parameter of `Combine` can be `dyn Any-ish`-erased,
  - and we'll strive to make it so the former (`'u`) be kept around in the type system at all times, _invariantly_.

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
https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=8450bb3525f5705ff34cce6730d5c1ce)**
<details><summary>Click here to fiddle with the full snippet inline</summary>

```rust ,edition2018,editable
{{#include naive_any_example.rs:all}}
```

</details>

___

## Limitations of this design

So, the previous `Put<'lt> & Remove<'lt>` couple-traits design is indeed quite nifty since:

  - it expresses the key `&'u i32 = Combine<'u, &'static i32>` intuition, which is both intuitive, and (thus)
    less error-prone to work with;

  - all within a _non-`unsafe`_ `trait` design capable of expressing all this accurately and
    soundly;

  - in a rather "standalone" fashion insofar it does not involve using, and thus knowing about,
    higher-kinded types.

So, in certain scenarios, _especially as a private-to-the-crate helper_, this approach can be the
best one, and the one exposed in the following section may not be necessary.

However, it does have a rather big drawback, one which grows bigger and worse the more you work with
this:

  - whilst it _appears_ to be a fully general solution,
  - in practice, however, it does require concrete impls for each possible lifetime-generic type you
    or downstream users may want to use! ⚠️

      - (for instance, notice how we had to write separate `impl`s for `&T` and `Cell<&T>`)

This last point leads to two annoying aspects:

  - <details open><summary><b>Problems of Coherence/the "orphan rules"</b></summary>

    This is most notoriously known as "the `serde` problem" (regarding which `::serde` in particular
    does nothing wrong, but for providing a very pervasive _trait_ to the Rust ecosystem altogether).

    The issue lies in the interaction between a non-`std`lib type, and a non-`std`lib trait, when
    both stem from distinct crates:

      - "the orphan rules" dictate that the `impl` of `ThatTrait for ThatType` lie in either of these two crates.

      - the problem is, these two crates may not know of each other! They may just happen to end up
        put together by a third-party dependent crate, which wants such usage. And this third party
        dependent cannot write the `impl`, since the orphan rules forbid it. So they are stuck, but
        for having to write a newtype-wrapper to soothe the Almighty Coherence.

      - the other option is for one of these two crates to kind of artificially make itself aware of
        the other (_c.f._ `serde` features on many crates out there); but this only works when one of these two crates is famous enough to warrant
        such a dedicated support by the other crate.

    That is, **this design goes against organic composition of libraries**.

    Back to our `Put & Remove` trait(s), which we will deem non-"famous enough", it means it is up
    to us to think of types for which `Put` may want to be implemented. So let's think of
    lifetime-infected types, be it in the stdlib, or in notorious Rust crates:

      - <code>[::std::borrow::Cow]\<\'lt, …\></code>
      - <code>[::std::fmt::Formatter]\<\'lt\></code>
      - <code>[::std::os::fd::BorrowedFd]\<\'lt\></code>
      - <code>[::std::cell::Ref]\<\'lt, …\></code>
      - <code>[::std::cell::RefMut]\<\'lt, …\></code>
      - <code>[::std::sync::RwLockReadGuard]\<\'lt, …\></code>
      - <code>[::std::sync::RwLockWriteGuard]\<\'lt, …\></code>
      - <code>[::std::sync::MutexGuard]\<\'lt, …\></code>
      - <code>[::tokio::sync::RwLockReadGuard]\<\'lt, …\></code>
      - <code>[::tokio::sync::RwLockWriteGuard]\<\'lt, …\></code>
      - <code>[::tokio::sync::MutexGuard]\<\'lt, …\></code>
      - <code>[::pyo3::marker::Python]\<\'lt\></code>

    [::std::borrow::Cow]: https://doc.rust-lang.org/stable/std/borrow/enum.Cow.html
    [::std::os::fd::BorrowedFd]: https://doc.rust-lang.org/stable/std/os/fd/struct.BorrowedFd.html
    [::std::fmt::Formatter]: https://doc.rust-lang.org/stable/std/fmt/struct.Formatter.html
    [::std::cell::Ref]: https://doc.rust-lang.org/stable/std/cell/struct.Ref.html
    [::std::cell::RefMut]: https://doc.rust-lang.org/stable/std/cell/struct.RefMut.html
    [::std::sync::RwLockReadGuard]: https://doc.rust-lang.org/stable/std/sync/struct.RwLockReadGuard.html
    [::std::sync::RwLockWriteGuard]: https://doc.rust-lang.org/stable/std/sync/struct.RwLockWriteGuard.html
    [::std::sync::MutexGuard]: https://doc.rust-lang.org/stable/std/sync/struct.MutexGuard.html

    [::tokio::sync::RwLockReadGuard]: https://docs.rs/tokio/1.32.0/tokio/sync/struct.RwLockReadGuard.html
    [::tokio::sync::RwLockWriteGuard]: https://docs.rs/tokio/1.32.0/tokio/sync/struct.RwLockWriteGuard.html
    [::tokio::sync::MutexGuard]: https://docs.rs/tokio/1.32.0/tokio/sync/struct.MutexGuard.html

    [::pyo3::marker::Python]: https://docs.rs/pyo3/0.19.2/pyo3/marker/struct.Python.html

    😵‍💫😵‍💫😵‍💫 And so on and so forth 😵‍💫😵‍💫😵‍💫

    </details>

  - <details open><summary><b>The case of <code>T<'static></code></b></summary>

    Consider, for instance, the type `&'static str`.

    What should `<&'static str as Put<'lt>>::Infected` be?

      - On the one hand, we could be tempted to say that it should be `&'lt str`;
      - But it would be just as legitimate to say that it could stay as `&'static str`.

    To justify that second point, consider wanting to coërce together, under a type-unifying
    `MyAny<'lt>`, a value `x: &'lt i32`, and some `"string literal": &'static str`.

    When `<&'static str as Put<'lt>>::Infected = &'lt str`, this means that technically we cannot
    put our `&'static str` behind a `MyAny<'lt>`!

    > But `&'_ str` is covariant over `'_`— you may retort—, so we can shrink our `&'static str`
    > down to `&'lt str` so as to match `MyAny<'lt>`, can't we?

    True! And that does work, but this "lifetime shrinkage":

      - does result in a loss of information about our `&str` borrowing a never-dangling string
        literal (<code>&<span style="color: red;">\'static</span> str</code>);

        Which, in turn, means that we'll only be able to extract `&'lt str`s out of our
        `MyAny<'lt>`s: any long-lived usage of the string in question would then require
        `.to_owned()`-ing it…

      - is not possible for non-covariant types, such as `Mutex<&'static str>`.

    ```rs ,ignore
    // 1. Let's say this choice had been made…
    impl<'lt> Put<'lt> for &str {
        type Infected = &'lt str;
    }

    let local = 42; // 'lt => &local.

    let mut anys: Vec<Box<dyn MyAny<'_>>> = vec![coërce(&local)];

    let some_str_literal = "I live forever!";
    anys.push(coërce(some_str_literal));

    // 2. How do we make a `&'static str` "remember" its `'static`-ness when
    //    `MyAny<'lt>`-erased?
    'later: {
        // Error, expected a `&'static str`, got a `&'lt str`!
        // (to clarify: got a `<&'static str as Put<'lt>>::Infected = &'lt str`).
        let some_str_literal: &'static str = anys[1].downcast_ref::<&str>().unwrap();
    }

    // 3. Or even just work with some invariant type such as `Mutex<&'static str>`?
    let mutexed = ::std::sync::Mutex::new(some_str_literal);
    // Error, expected a `Mutex<&'lt str>`, got a `Mutex<&'static str>`!
    anys.push(coërce(mutexed));
    ```

    So, can either choice be made?

    Yes, but this choice can only be made _once and for all_, since it has to be engraved in
    `impl trait` stone.

    </details>

Eventually, both issues end up leading to the need for cumbersome newtype wrappers, should we want
to override (or even just _provide_) the `impl Put<'lt>` we need/want:

```rs
// 1. Let's say this choice had been made…
impl<'lt> Put<'lt> for &str {
    type Infected = &'lt str;
}

let local = 42; // 'lt => &local.

let mut anys: Vec<Box<dyn MyAny<'_>>> = vec![coërce(&local)];

let some_str_literal = "I live forever!";

// 2. Solution:
pub struct Wrapper<T>(pub T);

/// Let's say that this `Wrapper` just disregards the given `'lt`.
impl<'lt, T> Put<'lt> for Wrapper<T> {
    type Infected = Wrapper<T>;
}

anys.push(coërce(Wrapper(some_str_literal)));
'later: {
    // OK ✅
    // (to clarify: got a `<W<&'static str> as Put<'lt>>::Infected = W<&'static str>`).
    let Wrapper(some_str_literal): Wrapper<&'static str> = *anys[1].downcast_ref().unwrap();
}

// OK ✅
// (since `Wrapper<Mutex<&'static str>> : Put<'lt, Infected = … 'static …>`)
let mutexed = ::std::sync::Mutex::new(some_str_literal);
anys.push(coërce(Wrapper(mutexed)));
```

___

So, is there a way to design something similar to our `Put<'lt> & Remove<'lt>`-powered `MyAny`, but
for not necessarily relying on _concrete_, _specific_ and hard-coded `impl`s?

**Some kind of blanket impl over all sorts of borrows, and some syntax to let us disambiguate the one
we want?**

_e.g._, some way to disambiguate `as Put<'lt>>::T = &'lt str` _vs._ `as Put<'lt>>::T = &'static str`.

In pseudo-code, and after having removed the `as Put` and `::T` noise:

  - disambiguating between `<'lt> = &'lt str` and `<'lt> = &'static str`.

Well, these do look a lot like our `ForLt!(<'lt> = &'lt str)` _vs._ `ForLt!(<'lt> = &'static str)`
(arrow-kinded) types, do they not?

Does that mean we can make `ForLt!`-powered (HKT) `dyn MyAny<'lt>` design work?
