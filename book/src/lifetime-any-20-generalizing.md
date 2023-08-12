# Fully generalizing this pattern 🤯

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
