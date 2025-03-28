//! Niche and advanced, you can ignore this to begin with.
//!
//! Module for other [`ForLt!`][ForLt!]-like constructs, or for
//! [`ForLt`][trait@ForLt]-adjacent definitions, such as [`WithLifetime`].

use super::ForLt;

pub
mod extra_arities;

pub
mod type_eq;

/// The [`::nougat`](https://docs.rs/nougat) of this crate, which allows "expressing `dyn`-safe
/// <code>impl [ForLt][trait@ForLt]</code> types", sort to speak.
///
/// The idea is that in order for the <code>: [ForLt][trait@ForLt]</code> design to be ergonomic,
/// it is paramount to have a [`ForLt!`]-like companion construct, in the same fashion that in order
/// for the stdlib [`Fn…`][FnOnce] traits to be ergonomic and useful, it is paramount to have
/// literal `|…| { … }` closures expressions as the dual construct.
///
/// And the trick to achieve producing an, on-the-fly, `impl ForLt` type, _i.e._, an, on-the-fly,
/// type which `for<'any_lt>`, associates a type to such `'any_lt`, is to involve a
/// `dyn for<'any_lt> SomeTrait<'any_lt, Assoc = …>` of some sorts.
///
/// [`WithLifetime`] is such a `SomeTrait`.
///
/// Actual usefulness of this trait for downstream users is to be seen, but since there seemed to be
/// some interest, let's expose it and see how users end up using it.
pub
trait WithLifetime<'lt>
:
    Send + Sync + Unpin
{
    type T;
}

#[cfg(not(feature = "fn_traits"))]
impl<'lt, T : ?Sized + WithLifetime<'lt>>
    WithLifetime<'lt>
for
    crate::ඞ::ForLt<T>
{
    type T = T::T;
}

/// The key connection between [`ForLt`][trait@ForLt] and [`WithLifetime`].
impl<T : ?Sized> ForLt for T
where
    Self : for<'any> WithLifetime<'any>,
{
    type Of<'lt> = <Self as WithLifetime<'lt>>::T;
}
