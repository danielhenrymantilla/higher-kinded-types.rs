# The following snippets fail to compile

## `For!` requires `new_For_type!` around it

```rust ,compile_fail
use ::higher_kinded_types::prelude::*;

type Vec_ = For!(<T> = Vec<T>);
```

<!-- Templated by `cargo-generate` using https://github.com/danielhenrymantilla/proc-macro-template -->
