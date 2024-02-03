
Let's circle back to something I promised, shall we?

Remember this?

 -  ```rust ,ignore
    {{#include forlt_any_example.rs:just-split}}
    ```

Let's focus on:

> There are a **couple edge cases** that make this technically **unsound** \[…\] we will revisit this point, and actually extract yet another **meaningful pattern off this observation** 💡

# Unsoundness of `carcass: Body::Of<'static>`

How is the above API _unsound_? In Rust, we say that an API is _unsound_ if it is possible, with non-`unsafe` Rust[^or_unsafe], to cause UB / break havoc with it.

[^or_unsafe]: or with `unsafe` Rust which satisfies the `// SAFETY` preconditions / narrow contract of said API

Which means that proving an API to be unsound is quite simple, and unquestionable: just produce such an exploit!

Now, to avoid spoiling the fun, I shan't be giving the answer right away, so as to let _you_ think a bit more about it.

  - Remember: when talking about the soundness (or lack thereof) of an API, what matters is said A**P**I, _i.e._, the `pub`lic `fn`s, fields, methods, `trait` implementations, _etc._

  - `fn into_inner()` is not technically needed for the exploit, so it's not the main culprit behind the unsoundness. But it can make writing the exploit prettier, hence my leaving it.

Here are **some hints** (partial spoilers ⚠️) to make this task easier:

 1. <details><summary>Click to reveal</summary>

      - The main `unsafe` operation here is the `transmut`ing of `'soul -> 'static` lifetimes in the `fn soul_split()` construction.

      - The "reasoning" behind the so-claimed "SAFETY" of that operation was the lack of exposure of the inaccurate `Body::of::<'static>` to "the outside world" / to public API.

      - Maybe that claim was too bold and some exposure slipped through the cracks / leaked?

    </details>

 1. <details><summary>Click to reveal</summary>

      - Did someone say _API leakage_?

      - What typical API property other than the pure `fn`s written above may be affected by the choice of field types?

    </details>

 1. <details><summary>Click to reveal</summary>

      - Adding a `_phantom: PhantomData<*mut ()>` field to the type solves the issue at which the previous bullet was hinting at.

    </details>

 1. <details><summary>Click to reveal</summary>

      - The _very astute_ reader may be able to come up with _another_ more subtle kind of API leakage (_i.e._, one which the `PhantomData<*mut ()>`, alone, does not fix), which also entails a soundness problem.

    </details>

 1. <details><summary>Click to reveal</summary>

      - I'll drop it there.

    </details>
