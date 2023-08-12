<span style="text-align: center;">

# Higher-Kinded Types

</span>

_Higher-Kinded Types_ is the broader term used to talk of _nested generics_,
_e.g._, having a type taking a generic parameter which would be, in turn,
generic:

```rust ,ignore
//! Pseudo-code!

/// Over-types "nested genericity"
//                             👇
struct HktExample<Collection : <T>> {
    ints: Collection<i32>,
    strings: Collection<String>,
}

/// Over-lifetimes "nested genericity"
//                               👇
struct HktExampleLt<'a, 'b, T : <'_>> {
    a: T<'a>,
    b: T<'b>,
}
```

From there, we can see that the key component / the keystone allowing for
Higher-Kinded APIs to be expressible, is that of the "nested genericity".

  - In Haskell parlance, we could call them "Arrow-Kinded Types":

      - The kind of `T` in `T : <'_>` would be `'* -> *`.

        They express the property of "being generic".

      - HKTs would then be a "nestedly Arrow-Kinded" type:

        The kind of `HktExampleLt` would be `('* -> *) -> *`.

        They express the propery of "being generic over an itself generic type".

The [`::higher-kinded-types`](https://docs.rs/higher-kinded-types) crate on
which this book is based tries to remain close to Rust terminology, and these
"arrow-kinded types" are therein called "`For` types", given the higher-order-ish
nature of the nested generic parameter, not totally unlike `for<'lifetime> fn…`
pointers or `dyn for<'lifetime> Trait…`s.

## `For` types (such as `impl ForLifetime` types)

A "`For` type" is an actual / **fully standalone** type which is, itself,
"generic", or rather, to which we can further feed generic parameters (such as
lifetime parameters or type parameters) to obtain further types.

  - [ ] "is generic" / can be fed generic parameters to construct a type ❓
  - [ ] is a type in and of itself ❓
      - For instance, `type Standalone = YourType;` has to compile.

One way to illustrate this difference, for instance, would be to consider:

```rust ,ignore
use ::higher_kinded_types::{ForLifetime as ForLt};

type StrRefNaïve<'lt> = &'lt str;
// and
type StrRef = ForLt!(<'lt> = &'lt str);
```

Both `StrRefNaïve` and `StrRef` can be fed a generic parameter (in this
instance, a lifetime parameter) so as to get or construct a type:

```rust ,ignore
use ::higher_kinded_types::{ForLifetime as ForLt};

# type StrRefNaïve<'lt> = &'lt str;
# type StrRef = HKT!(<'lt> = &'lt str);
#
const _: StrRefNaïve<'static> = "This is a `&'static str`";
const _: <StrRef as ForLt>::Of<'static> = "This is a `&'static str`";
```

  - [x] "is generic" / can be fed generic parameters to construct a type ✅

But what of:

  - [ ] is a type in and of itself ❓

Well, while `StrRef` is indeed a standalone type:

```rust ,ignore
use ::higher_kinded_types::{ForLifetime as ForLt};

type StrRef = ForLt!(<'lt> = &'lt str);

type Standalone = StrRef; // ✅
```

it turns out that `StrRefNaïve` is not:

```rust ,compile_fail
type StrRefNaïve<'lt> = &'lt str;

type Standalone = StrRefNaïve; // ❌ Error
```

This errors with:

```console
error[E0106]: missing lifetime specifier
 --> src/higher_kinded_types.rs:70:19
  |
8 | type Standalone = StrRefNaïve; // ❌ Error
  |                   ^^^^^^^^^^^ expected named lifetime parameter
  |
help: consider introducing a named lifetime parameter
  |
8 | type Standalone<'a> = StrRefNaïve<'a>; // ❌ Error
  |                ++++   ~~~~~~~~~~~~~~~
```

That is, in Rust **a generic "type" is actually not a type**. It's just a "name"
(grammar-wise, it is called a "path"), to which we can feed the generic
parameters so as to _then_ obtain types.

A "`For` type" is the proper solution to this: not only can such an "entity" be
fed generic parameters (thence "acting like" a generic "type" above), it can
also _not be fed any parameters and still be a type_. That is,

> <span style="font-size: large;">a "`For` type" is an _actual_ **type** which is generic / to which we can feed parameters.</span>

## From `For` types to HKTs

We have this seemingly arbitrary definition just above precisely to allow us to
write actual HKT APIs, like the basic example at the beginning of this post:

```rust ,ignore
//! Pseudo-code

struct HktExampleLt<'a, 'b, T : <'_>> {
//                          👆 1
    a: T<'a>,
//      👆 2
    b: T<'b>,
//      👆 2
}
```

Notice how we need for `T` to be:

 1. A standalone/turbofishable type,
 2. to which we can feed generic parameters

Hence the `ForLifetime` abstraction showcased above!

```rust ,ignore
//! Real code! 🥳
use ::higher_kinded_types::{ForLt as Ofᐸᑊ_ᐳ};

struct HktExampleLt<'a, 'b, T : Ofᐸᑊ_ᐳ> {
    a: T::Of<'a>,
    b: T::Of<'b>,
}
```

  - I have used non-ASCII characters in order to rename `ForLt` as `Ofᐸᑊ_ᐳ`,
    just for the sake of illustrating the transition from the pseudo-code to the
    real code, since `Ofᐸᑊ_ᐳ` hopefully looks quite a bit like `Of<'_>`, thereby
    illustrating the actual usage we can make with these types: feeding them
    some lifetime `'x` parameter through the associated `Of<'x>` type (yes, this
    is a GAT).

    I won't be doing that anymore, since real code should not be using these
    `unicode_confusables`; I shall thenceforth only be using proper fully ASCII
    names such as `ForLt`:

    ```rust ,ignore
    //! Real code! 🥳
    use ::higher_kinded_types::ForLt;

    struct HktExampleLt<'a, 'b, T : ForLt> {
        a: T::Of<'a>,
        b: T::Of<'b>,
    }
    ```

So far these two snippets have illustrated how the `2`nd bullet of "being able
to feed a (lifetime) generic parameter to `T`" does indeed work, thanks to the
associated `Of<'_>` type of the `ForLt` trait (a GAT).

But what about the `1`st bullet of "being a standalone type"? We definitely run
into such a need the moment we try to turbofish and instantiate this
`HktExampleLt`:

```rust ,ignore
//! Real code! 🥳
#![forbid(elided_lifetimes_in_paths)]

use ::higher_kinded_types::ForLt;

struct HktExampleLt<'a, 'b, T : ForLt> {
    a: T::Of<'a>,
    b: T::Of<'b>,
}

type StrRefNaïve<'lt> = &'lt str;
// and
type StrRef = ForLt!(<'lt> = &'lt str);

let [a, b] = [String::from("a"), "b".into()];

let ok = HktExampleLt::<StrRef> {
    a: &a,
    b: &b,
};

#[cfg(this_would_error)]
let err = HktExampleLt::<StrRefNaïve> {
//                                 👆 error, missing `<'lifetime>` parameter
    a: &a,
    b: &b,
};

// Let's say that in this example neither the lifetime of `a` nor that of `b`
// "outlives" the other.
if ::rand::random() {
    drop(a);
    println!("{}", ok.b); // works thanks to distinct lifetimes!
} else {
    drop(b);
    println!("{}", ok.a); // works thanks to distinct lifetimes!
}
# println!("✅");
```

  - Notice how the `let err` case would not work, because of the missing
    lifetime parameter that `StrRefNaïve` requires _eagerly_ / first, which in
    our example does not make any sense: it is up to `a` and `b` to be picking
    their own lifetimes, which may be distinct, so `StrRefNaïve` has no reason
    to be picking _one_ upfront!

  - It is true, however, that for the sake of ergonomics, certain people have
    written libraries with this `ForLt` pattern smuggled under the hood, but
    with one important difference: rather than an explicit `ForLt!(&str)` type,
    what they have done is implementing their own flavor of the `ForLt` trait
    for "normal" Rust types (such as `&'ignored str`) directly:

    ```rust ,compile_fail
    //! What other crates often write for their own trait/API.

    impl<'completely_irrelevant> ForLifetime for &'completely_irrelevant str {
        type Of<'lt> = &'lt str;
    }

    // That way, they get to be able to write:
    let x = HktExampleLt::<&str> {
    //                     👆 Rust is picking some lifetime here, which is
    //                        unconstrained; in practice it will probably be 'static.
        a: &a,
        b: &b,
    };
    ```

    But, as we will see in [the section about lifetime-infected `Any`s](
    ./lifetime-any-hkt.md), this seemingly more appealing design is unable to
    express the distinction between `ForLt!(<'lt> = &'lt str)` and
    `ForLt!(<'lt> = &'static str)`, which can become a very unintuitive and
    frustrating limitation.
