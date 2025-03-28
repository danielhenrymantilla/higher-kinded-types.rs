//! Niche and advanced, you can ignore this to begin with.
//!
//! Module for other [`ForLt!`]-like constructs, or for
//! [`ForLt`]-adjacent definitions, such as [`WithLifetime`].
//!
//! [`ForLt`]: trait@crate::ForLt
//! [`ForLt!`]: crate::ForLt!

use super::*;

pub
mod extra_arities;

pub
mod type_eq;

/// The [`::nougat`](https://docs.rs/nougat) of this crate, which allows "expressing `dyn`-safe
/// <code>impl [ForLt]</code> types", sort to speak.
///
/// The idea is that in order for the <code>: [ForLt]</code> design to be ergonomic,
/// it is paramount to have a [`ForLt!`]-like companion construct, in the same fashion that in order
/// for the stdlib [`Fn…`][FnOnce] traits to be ergonomic and useful, it is paramount to have
/// literal `|…| { … }` closures expressions as the dual construct.
///
/// And the trick to achieve producing an on-the-fly `impl ForLt` type, _i.e._, an on-the-fly
/// type which associates, `for</* any */ 'lifetime>`, a type to such `'lifetime`, is to involve a
/// `dyn for<'any_lt> SomeTrait<'any_lt, Assoc = …>` of some sorts.
///
/// [`WithLifetime`] is such a `SomeTrait`.
///
/// Actual usefulness of this trait for downstream users is to be seen, but since there seemed to be
/// some interest, let's expose it and see how users end up using it.
///
/// [ForLt]: trait@ForLt
pub
trait WithLifetime<'lt>
:
    Send + Sync + Unpin
{
    type Of : ?Sized;
}

#[cfg(not(feature = "fn_traits"))]
impl<'lt, T : ?Sized + WithLifetime<'lt>>
    WithLifetime<'lt>
for
    crate::ඞ::ForLt<T>
{
    type Of = T::Of;
}

/// Same as [`ForLifetime`], but for having a `: ?Sized` "unbound" on its [`Of<'_>`][Self::Of]
/// associated type.
///
/// ## Relation to [`ForLifetime`]
///
/// There is currently none whatsoever (no blanket impl nor super-trait). This has been done on
/// purpose to avoid hindering the ergonomics of [`ForLifetime`] too much, which remains the
/// favored/preferred/deemed-more-useful flavor of the two traits.
///
/// There is, however, one connecting thing and redeeming factor: [`ForLt!`].
///
/// Indeed, the [`ForLt!`] macro produces a type which implements the necessary
/// <code>for\<\'any\> [WithLifetime]\<\'any\></code> which thereby makes it satisfy _both_
/// [`ForLifetime`] and [`ForLifetimeMaybeUnsized`] (assuming the type in it to be [`Sized`], else it
/// will only satisfy the latter, obviously).
///
/// This means that when dealing with an opaque <code>T : [ForLifetime]</code>, for instance,
/// one can use the type <code>[ForLt!]\(T::Of\<\'_\>\)</code> to get "back" a type which behaves
/// like `T`, but which happens to implement both traits.
///
/// ```rust
/// use ::higher_kinded_types::{ForLt, ForLifetime, advanced::ForLifetimeMaybeUnsized};
///
/// fn generic_over_both<T : ForLifetimeMaybeUnsized>() {}
///
/// fn generic_over_sized<T : ForLifetime>() {
///     generic_over_both::<ForLt!(T::Of<'_>)>();
/// }
///
/// generic_over_sized::<ForLt!(&str)>();
/// ```
///
///   - and _vice versa_:
///
///     ```rust
///     use ::higher_kinded_types::{ForLt, ForLifetime, advanced::ForLifetimeMaybeUnsized};
///
///     fn generic_over_maybe_unsized<T : for<'any> ForLifetimeMaybeUnsized<Of<'any> : Sized>>() {
///         generic_over_sized::<ForLt!(T::Of<'_>)>();
///     }
///
///     fn generic_over_sized<T : ForLifetime>() {
///     }
///
///     generic_over_maybe_unsized::<ForLt!(&str)>();
///     ```
pub
trait ForLifetimeMaybeUnsized : crate::seal::WithLifetimeForAny {
    /// "Instantiate lifetime" / "apply/feed lifetime" operation:
    ///
    ///   - Given <code>\<T : [ForLifetimeMaybeUnsized]\></code>,
    ///
    ///     `T::Of<'lt>` stands for the HKT-conceptual `T<'lt>` type.
    type Of<'lt> : ?Sized;
}

/// The key connection between [`ForLt`] and [`WithLifetime`].
///
/// [`ForLt`]: trait@crate::ForLt
impl<T : ?Sized> ForLifetime for T
where
    Self : for<'any> WithLifetime<'any, Of : Sized>,
{
    type Of<'lt> = <Self as WithLifetime<'lt>>::Of;
}

impl<T : ?Sized> ForLifetimeMaybeUnsized for T
where
    Self : for<'any> WithLifetime<'any>,
{
    type Of<'lt> = <Self as WithLifetime<'lt>>::Of;
}
