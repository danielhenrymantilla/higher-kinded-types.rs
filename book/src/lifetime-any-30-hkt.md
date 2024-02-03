# Simple non-`'static` `dyn Any` with HKTs

So, how can we reïmplement the previous design, but replacing `Put<'r> & Remove<'r>` with
`ForLt!`s?

## `type Combine<'r, X> = …`

Well, remember that the key intuition we used previously was this `&'r str = Combine<'r, Smth>` "split" (indeed, if `Smth` is `: 'static`, then we can erase it as a dynamic `TypeId`, and we can get our type back by virtue of `Combine`-ing it with our (non-erased) `'r`):

### With `Put<'r> & Remove<'r>`

```rust ,ignore
mod put_trait {
    use …::Put;
    pub type Combine<'r, X : Put<'r>> = <X as Put<'r>>::T;
    //                   ^^^^^^^^^^^
    //                   limitation: not all types impl `Put<'r>`
}
```

so that:

```rust ,ignore
put_trait::Combine<'r, &'static str>
    = <&'static str as Put<'r>>::T
    =  &'r str
```

### With `ForLt!`

```rust ,ignore
mod hkt {
    use …::ForLt;

    pub type Combine<'r, X : ForLt> = X::Of<'r>;
    //                   ^^^^^^^^^^^
    //                   limitation: requires `ForLt!` macro to use.
    //                   benefit: can handle *all* the `<'r>`-generic types!!
}
```

so that:

```rust ,ignore
hkt::Combine<'r, ForLt!(&str)>
    = <ForLt!(&'_ str) as ForLt>::Of<'r>
    =         &'r str
```

#### Benefits of this design

Remember the `&'static str = Combine<'r, ???>` [_conundrum_ of the `Put<'r>` design](lifetime-any-20-generalizing.md#limitations-of-this-design)? Well, with `Combine<'lt, X : ForLt> = X::Of<'lt>`, we have a **simple solution** to this:

 1. instead of using `X = ForLt!(&'_ str)` (_i.e._, `X = ForLt!(<'r> = &'r str)`),
 1. we can simply use `X = ForLt!(&'static str)` (_i.e._, `X = ForLt!(<'r> = &'static str)`):

```rust ,ignore
hkt::Combine<'r, ForLt!(&'static str)>
      = <ForLt!(<'r> = &'static str) as ForLt>::Of<'r>
      = &'static str // ✅
```

And this solution is also **scalable**, insofar it is able to perfectly tackle arbitrarily complex types such as `Cow<'r, str>`, `fd::Borrowed<'r>`, _etc._:

```rust ,ignore
/// Find X so that:
/// hkt::Combine<'r, X> = Cow<'r, str>
type X = ForLt!(Cow<'_, str>);
```

```rust ,ignore
/// Find X so that:
/// hkt::Combine<'r, X> = fd::Borrowed<'r>
type X = ForLt!(fd::Borrowed<'_>);
```

## `type T = Combine<'soul, Body>`

In fact, if we consider a type's "body" the `TypeId`-runtime-materializable part of a type, and its `'r`-dependent component its "soul", which cannot be materialized / reïfied within runtime / monomorphized data, then this `Combine` operator represents the act of imbuing a body with a soul; and in the other direction, the act of splitting a type `T` into its `'soul` and `Body` constituents:

```rust ,ignore
/// A soul-splitting operation
&'lt str = Combine<'lt, ForLt!(&'_ str)>;
                // ^^^  ^^^^^^^^^^^^^^^
                // soul      body
```

<span style="text-align: center;">

<img
    src="https://static.wikia.nocookie.net/dota2_gamepedia/images/d/dd/Parting_Shot_icon.png"
    height = "100px"
    title = "Dota: Parting Shot"
/>

</span>

And indeed, once equipped with such a tool/pattern, we are easily able to systematically perform this `dyn Any`-ification of arbitrary `<'lt>`-dependent types:

 1. ```rust ,ignore
    {{#include forlt_any_example.rs:split1}}
    ```

    ![](https://gist.github.com/assets/9920355/5ad1d163-d2f9-41d4-87ca-121cb82a7bf4)

    ```rust ,ignore
    {{#include forlt_any_example.rs:split2}}
    ```

 1. And once we have a `Split<'soul, Body>`, it's easy to see how the `Any`-fication of the body can take place:

    ```rust ,ignore
    /// Reminder.
    pub
    struct Split<'soul, Body : ForLt> {
        _soul: PhantomInvariant<'soul>,
        carcass: Body::Of<'static>,
    }
    {{#include forlt_any_example.rs:split-any-body}}
    ```

 1. Usage:

    ```rust ,ignore
    {{#include forlt_any_example.rs:main}}
    ```

**[Full snippet playground](https://play.rust-lang.org/?version=nightly&mode=debug&edition=2021&gist=37194cdb1cdf5587f72ab862879cedcc)**
<details><summary>Click here to play with the full snippet inline</summary>

```rust ,edition2018,editable
{{#include forlt_any_example.rs:all}}
```

</details>
