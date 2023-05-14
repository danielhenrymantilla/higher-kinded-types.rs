//! It turns out that `fn(Input<'_>) -> Output<'_>`, when you think about it,
//! is also:
//!   - a real Rust type;
//!   - which conceptually expresses the `HKT!(Output<'_>)` property:
//!
//! That is, one can feed a `'_` to it by feeding `Input<'_>` to the `FnOnce`
//! trait, and then querying the resulting `Output` type.
//!
//! Sadly, this only seems to work when using, _verbatim_, the real `FnOnce`
//! trait.
//!
//! That is, no amount of stable-polyfill _à la_:
//!
//! ```rust ,compile_fail
//! use ::higher_kinded_types::{*, ඞ::WithLifetime};
//!
//! type For<'lt> = &'lt ();
//!
//! /// coherence wrappers
//! struct A<F>(F);
//! struct B<F>(F);
//!
//! impl<'lt, F, R> WithLifetime<'lt> for A<F>
//! where
//!     F : FnOnce(For<'lt>) -> R,
//!     F : Send + Sync + Unpin,
//! {
//!     type T = R;
//! }
//!
//! impl<'lt, F> WithLifetime<'lt> for B<F>
//! where
//!     F : MyFnOnce<For<'lt>>,
//!     F : Send + Sync + Unpin,
//! {
//!     type T = F::Ret;
//! }
//!     trait MyFnOnce<A>
//!     where
//!         Self : FnOnce(A) -> Self::Ret,
//!     {
//!         type Ret;
//!     }
//!     impl<F, A, R> MyFnOnce<A> for F
//!     where
//!         Self : FnOnce(A) -> R,
//!     {
//!         type Ret = R;
//!     }
//!
//! fn test<Ret : HKT>(_: impl FnOnce(&str) -> Ret::__<'_>)
//! {}
//!
//! /// `HKT!(&str)`.
//! type StrRef = fn(For<'_>) -> &str;
//!
//! test::<A<StrRef>>(|s| s);
//! test::<B<StrRef>>(|s| s);
//! ```
//!
//! is able to make it work.
//!
/** They yield:
error: implementation of `WithLifetime` is not general enough
  --> src/fn_traits.rs:59:1
   |
48 | test::<A<StrRef>>(|s| s);
   | ^^^^^^^^^^^^^^^^^ implementation of `WithLifetime` is not general enough
   |
   = note: `A<for<'r> fn(&'r ()) -> &'r str>` must implement `WithLifetime<'0>`, for any lifetime `'0`...
   = note: ...but it actually implements `WithLifetime<'1>`, for some specific lifetime `'1`

error: implementation of `WithLifetime` is not general enough
  --> src/fn_traits.rs:60:1
   |
49 | test::<B<StrRef>>(|s| s);
   | ^^^^^^^^^^^^^^^^^ implementation of `WithLifetime` is not general enough
   |
   = note: `B<for<'r> fn(&'r ()) -> &'r str>` must implement `WithLifetime<'0>`, for any lifetime `'0`...
   = note: ...but it actually implements `WithLifetime<'1>`, for some specific lifetime `'1`
**/

use super::*;

pub
struct Input<'lt>(*mut Self);

impl<'lt, F> WithLifetime<'lt> for F
where
    F : FnOnce<(Input<'lt>, )>,
    F : Send + Sync + Unpin,
{
    type T = F::Output;
}
