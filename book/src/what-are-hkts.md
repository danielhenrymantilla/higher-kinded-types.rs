# What are HKTs

A higher-kinded type is an actual / **full / standalone** type which is, itself,
"generic", or rather, to which we can further feed generic parameters (such as
lifetime parameters or type parameters) to obtain further types.

  - [ ] "is generic" / can be fed generic parameters to construct a type ❓
  - [ ] is a type in and of itself ❓
      - For instance, `type Standalone = YourHktType;` has to compile.

One way to illustrate this difference, for instance, would be to consider:

```rust
use ::higher_kinded_types::Gat;

type StringRefNaïve<'lt> = &'lt str;
// and
type StringRef = Gat!(<'lt> = &'lt str);
```

Both `StringRefNaïve` and `StringRef` can be fed a generic parameter (in this
instance, a lifetime parameter) so as to get or construct a type:

```rust
use ::higher_kinded_types::Gat;

# type StringRefNaïve<'lt> = &'lt str;
# type StringRef = HKT!(<'lt> = &'lt str);
#
const _: StringRefNaïve<'static> = "This is a `&'static str`";
const _: <StringRef as Gat>::Of<'static> = "This is a `&'static str`";
```

  - [x] "is generic" / can be fed generic parameters to construct a type ✅

But what of:

  - [ ] is a type in and of itself ❓

Well, while `StringRef` is indeed a standalone type:

```rust
use ::higher_kinded_types::Gat;

type StringRef = Gat!(<'lt> = &'lt str);

type Standalone = StringRef; // ✅
```

it turns out that `StringRefNaïve` is not:

```rust ,compile_fail
type StringRefNaïve<'lt> = &'lt str;

type Standalone = StringRefNaïve; // ❌ Error
```

This errors with:

```console
error[E0106]: missing lifetime specifier
 --> src/higher_kinded_types.rs:70:19
  |
8 | type Standalone = StringRefNaïve; // ❌ Error
  |                   ^^^^^^^^^^^^^^ expected named lifetime parameter
  |
help: consider introducing a named lifetime parameter
  |
8 | type Standalone<'a> = StringRefNaïve<'a>; // ❌ Error
  |                ++++   ~~~~~~~~~~~~~~~~~~
```

That is, in Rust **a generic "type" is actually not a type**. It's just a path
(grammar-wise), a name, to which we can feed the generic parameters so as to
obtain types in return.

A HKT would be the proper solution to this: not only can such an "entity" be
fed generic parameters (thence "acting like" a generic "type" above), it can
also _not be fed any parameters and still be a type_. That is,

> <span style="font-size: large;">a HKT is an _actual_ **type** which is generic / can be fed parameters.</span>

Another definition, which will make more sense in the following section, is that
HKTs come into play the moment we need "generic generics".
