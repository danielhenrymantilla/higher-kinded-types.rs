> 1. <details open><summary>Click to reveal</summary>
>
>      - The _very astute_ reader may be able to come up with _another_ more subtle kind of API leakage (_i.e._, one which the `PhantomData<*mut ()>`, alone, does not fix), which also entails a soundness problem.
>
>    </details>
>
> 1. <details open><summary>Click to reveal</summary>
>
>      - I'll drop it there.
>
>    </details>

![mic drop](https://gist.github.com/assets/9920355/7d137a08-95f5-4e8f-b3d0-bf73e52cf4a9)

# Exploiting the unsoundness, method 2

So, for this one I'm going to take the inverse approach to that of the previous section, and, instead, start with a fully working exploit, and explain what is going on from there, shall we?

## The exploit

```rust ,ignore
use ::std::sync::{Mutex, MutexGuard};

/// type ForMutexGuard::Of<'mutex> = MutexGuard<'mutex, ()>;
type ForMutexGuard = ForLt!(MutexGuard<'_, ()>);

fn exploit() {
    let mutex = Mutex::new(());
    // `'_` refers to the `'mutex`.
    let lock_guard: MutexGuard<'_, ()> = mutex.lock().unwrap();
    let lock_guard: Animaterium<'_, ForMutexGuard> = soul_split(lock_guard); // 👈
    drop(mutex);
} // <- `lock_guard` dropped here, releasing a dropped mutex!??
```

This snippet does indeed compile, unless the `lock` shadowing line is commented out.

To better describe what has been happening, consider what the `'mutex` inside the `lock_guard` is after the `drop(mutex);`: since the referee `mutex` has been dropped, our `lock_guard` reference is said to be `'dangling`, which is how we are going to name such a lifetime:
  - A `MutexGuard<'dangling, ()>` which "falls out of scope" (and is thus _implicitly_ dropped) is **not allowed** / is **soundly rejected** by the compiler, thereby avoiding the use-after-free issue.
  - But an `Animaterium<'dangling, ForMutexGuard>` "equivalent" data structure is **allowed to "fall out of scope"** (thus getting _implicitly_ dropped), with **no errors** or warnings whatsoever, resulting in a use-after-free, and thus, **Undefined Behavior**.

### Another example

Since it is hard, without Miri, to "showcase" such UB, let's re-write this problematic snippet in a way wherein the UB hopefully is more visible[^1]:

```rust ,edition2018
{{#include may_dangle_drop_exploit.rs:all}}
```

  - Don't forget to click the <i class="fa fa-play play-button"></i> button to demo it!

**[Full snippet Playground](https://play.rust-lang.org/?version=nightly&mode=debug&edition=2021&gist=f1485ff81777ad54ca5cc9e8083a51a2)**

<details><summary>Click here to fiddle with the full snippet inline</summary>

```rust ,edition2018,editable
{{#include may_dangle_drop_exploit.rs:all}}
```

</details>

[^1]: _expecting_ UB to behave in some specific way is an oxymoron, but in practice, the tests on the current version of the Playground have quite reliably showcased it. YMMV.

## Explanation

### Pre-requisite: `dropck`

The behavior showcased here, illustrates a rather subtle —albeit important!— aspect of Rust: **"borrow-checking implicit drop points"**, we could call this. It is also more commonly called drop-checking, or just **`dropck`**, and more _specifically_, around drop-checking a value whose type may be carrying some **`'dangling` lifetime**.

To illustrate, consider:

 1. ```rust ,edition2018
    {
        let mut s = String::from("…");
        let r = &mut s;
        drop(s);
    } // <- `r` falls out of scope, even though `s` has been dropped.
    ```

    This is allowed since `r: &'dangling mut String`, is "just a reference": it has no drop glue whatsoever, so its `'dangling`-ness when it does out of scope is harmless.

      - This behavior is also dubbed "NLL", since the initial borrow-checker of Rust would actually reject such snippets! When it got upgraded to handle these scenarios, the added heuristic, and, metonimically, the borrow-checker itself, got called "non-lexical lifetimes".

 1. ```rust ,edition2018
    {
        let s = String::from("…");
        let r = (String::from("hi"), &s);
        drop(s);
    } // <- `r` falls out of scope, even though `s` has been dropped.
    ```

    This one is a bit harder to justify, or at least, the previous reasoning, alone, does not suffice.

    Indeed, this time we have a _tuple_ `r: (String, &'dangling String)`. And since `String` has drop glue[^nit], so does `r`, so the argument "no drop glue thus no harm possible" no longer suffices.

    The actual reasoning now needs to involve a field-per-field (and transitively onwards) analysis:

      - let's see _which_ fields specifically involve the `'dangling` lifetime;
      - and recursively `dropck` these.

    The "fields" of `r` are then `String`, which involves no lifetime, and `&'dangling String`, which has no drop glue (we are back to `1.`).

[^nit]: …even though there is no `impl Drop for String`. Indeed, `impl Drop` is rather `impl ExtraDropGlue`. For a type such as `Vec`, which is made of a raw pointer (with no drop glue), and a `len: usize` and `cap: usize` fields with no drop glue either, an explicit `impl ExtraDropGlue for Vec<…> {` was needed, so as to convey the need to drop each of its items and then deällocate the backing buffer. But since `String` is just a wrapper around a `Vec<u8>`, it **sructurally inherits** such drop logic, so that the buffer backing the UTF-8 contents of the `String` are already automagically deällocated. No need for `ExtraDropGlue`. A pedantic nit, but an important one nonetheless.

 3. ```rust ,edition2018,compile_fail
    use Drop as ExtraDropGlue;

    struct Foo<'r>(String, &'r str);

    impl ExtraDropGlue for Foo<'_> {
        fn drop(&mut self) {}
    }

    {
        let s = String::from("…");
        let r = Foo(String::from("hi"), &s);
        drop(s);
    } // <- `r` falls out of scope, even though `s` has been dropped.
      //    => ERROR!
    ```

    This snippet is now rejected, because `Foo<'r>` has an explicit _and arbitrary_ `impl ExtraDropGlue for` it, and Rust then conservatively assumes that anything `'r`-dependent may now be reached and used inside the `fn drop(&mut self) { … }` body, so `'r` better not be `'dangling`!

 4. And now, the last piece. This is rare to come by, but it does allow for some nicer ergonomics in Rust code here and there:

    ```rust ,edition2018
    {
        let s = String::from("…");
        let r = vec![(String::from("hi"), &s)];
        drop(s);
    } // <- `r` falls out of scope, even though `s` has been dropped.
    ```

    This snippet is "back" to being allowed, despite technically being the same as `Foo` above, except with `Foo` having been replaced with `Vec` (and `Vec` does have an explicit `impl ExtraDropGlue for` it)!

    We could argue "`std`lib magic", to which I'd say:

    ![No no, he's got a point](https://gist.github.com/assets/9920355/61b2c4f3-58dd-4a52-b398-9fe078f51fb2)

    <details open><summary>Click here to skip potentially overwhelming details</summary>

    But to be a bit more precise, what the stdlib has is:

    ```rust ,ignore
    use Drop as ExtraDropGlue;

    unsafe // <- !
    //       ????
    //   vvvvvvvvvvvvv
    impl<#[may_dangle] 'r>
        ExtraDropGlue
    for
        Vec<(String, &'r str)>
    {
    ```

    We can see this very odd-looking `<#[may_dangle] 'r>`, which is indeed a way to exceptionally pinky-promise (hence the `unsafe` in the `impl`) that,

      - whilst the body of that `fn drop(&mut self) { … }` may technically do arbitrary things and potentially reach `&'r str` entities,
      - in practice that shall not happen: no `'r`-depending instance shall be ~~harmed~~ dereferenced in the process of handling `Vec`'s own `ExtraDropGlue` (and since its fields have no extra drop glue either, we have successfully fully `dropck`-ed our `Vec`).

    Note that `Vec` can have any kind of item inside it, so, Rust has yet another odd-looking special syntax to generalize to any kind of `type T<'r> = …;` item type:

    ```rust ,ignore
    //! Pseudo-code
    use Drop as ExtraDropGlue;

    unsafe
    impl<#[may_dangle] 'r, T<'r>>
        ExtraDropGlue
    for
        Vec<T<'r>>
    {
    ```

      - in our case: `type T<'r> = (String, &'r str);`, for instance.

    If we now remove the renaming of `Drop`, use `rustfmt`, as well as the actual official syntax for these semantics, we do end up with the infamous:

    ```rust ,ignore
    unsafe impl<#[may_dangle] T> Drop for Vec<T> {
    ```

      - Again, it doesn't make sense to say, for a type, that it _dangles_, only lifetimes/references do. So `<#[may_dangle] T>` is magic syntax to state that _any lifetime_ `'x` appearing within `T`, is allowed to dangle when `dropck`ing a `Vec<T>`, …

        … on condition that an extra check be done: if `Self` now happens to "structurally own" an `'x`-infected field with its own drop glue, then the `dropck` logic needs to recurse and consider that field.

      - And `Vec<T>` does have such a field: `_owns_T: PhantomData<T>`, precisely for this purpose. Thus, in the case of `Vec<T>`, `dropck` needs to consider `T` itself.

        This may be a bit too abstract at this point, but consider having a `Vec<PrintOnDrop<'dangling>>`:

          - The `Vec`, in and of itself, is not going to use that `'dangling` lifetime (hence the `impl<#[may_dangle]`)

          - But the `Vec` will drop its own items, that is, it will drop `PrintOnDrop<'dangling>` instances. So `dropck` needs to be aware of this, and in this example, reject the code. This is what the `PhantomData` has achieved.

            Do note that `_onws_T: …` is only needed to begin with because of `#[may_dangle]`: only then does `PhantomData` play a meaningful role w.r.t. `dropck`.

    ___

    </details>

### Back to our `MutexGuard<'dangling, ()>`

There is an `impl ExtraDropGlue for MutexGuard<'_, …> {` in the stdlib, which means that `dropck` will never allow for that `'_` lifetime parameter to ever dangle, not even in out-of-scope / implicit-drop cases.

But _quid_ of / what about `Animaterium<'_, ForMutexGuard>`?

Let's see what the definition of that is, shall we?

 1. We have:

    ```rust ,ignore
    struct Animaterium<'soul, Body : ForLt> {
        _soul: PhantomInvariant<'soul>,
        carcass: Body::Of<'static>,
    }
    ```

 1. With `Body = ForLt!(MutexGuard<'_, ()>)`, this yields:

    ```rust ,ignore
    struct Animaterium<'mutex, Body = ForMutexGuard> {
        _soul: PhantomInvariant<'mutex>,
        carcass: MutexGuard<'static, ()>,
    }
    ```

 1. Now, consider `dropck`-ing a `'mutex = 'dangling` case:

     1. **`struct Animaterium<…>` has no explicit `impl ExtraDropGlue for` it**, so we cannot "bail early" with a rejection.

     1. Let's recursively `dropck` _each_ of its fields, then:

          - `_soul` has no drop glue whatsoever, so it can be ignored.

          - **`carcass`' type involves no `'mutex = 'dangling` whatsoever, so no need to worry about `'dangling` lifetimes here**.

Hence the problem.
