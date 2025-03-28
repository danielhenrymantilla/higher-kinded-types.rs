//! Module for type-equality hacks.
//!
//! In certain scenarios, you may have a `T : ForLt` in scope which you want to
//! constrain to match some specific type when fed some specific lifetime;
//! _e.g._
//!
//! ```rust ,ignore
//! //! pseudo-code
//! <'r, T : ForLt>
//! where
//!     T::Of<'r> = &'r str
//! ```
//!
//! In that case, you can actually just write the corresponding real code:
//!
//! ```rust
//! # use ::higher_kinded_types::prelude::*;
//! #
//! // Alas, for an equality bound using a _fixed_ rather than a `for<>` lifetime, naming
//! // the internal `ForLifetimeUnsized` super trait is necessary.
//! // Otherwise, the opaque `Of<'not_s> : Sized` type conflicts with `Of<'s> = &'s str`.
//! fn f<'s, T : ForLifetimeUnsized>(s: &'s str)
//! where
//!     T : ForLifetimeUnsized<Of<'s> = &'s str>,
//! {
//!     let _: T::Of<'s> = s;
//! }
//! ```
//!
//! which does work 🙂.
//!
//! It even works with higher-order lifetimes!
//!
//! ```rust
//! # use ::higher_kinded_types::ForLt;
//! #
//! fn f<T : ForLt>(s: String)
//! where
//!     T : for<'local> ForLt<Of<'local> = &'local str>,
//! {
//!     let _local: T::Of<'_> = &s;
//! }
//! ```
//!
//! But in other more contrived situations, potentially outside of HKTs
//! altogether, these type equality bounds may be unusable or buggy.
//!
//! In that case, the workaround is to replace type _equality constraints_, with
//! good old _trait bounds_, but with a trait designed so as to nonetheless
//! signify type equality:
//!
//! Hence the trait [`Is`]. Alas, it does come with some caveats, since when
//!
//! ```rust
//! # #[cfg(any)] macro_rules! ignore {
//! T : Is<EqTo = U>,
//! # }
//! ```
//!
//! Rust will nonetheless still be treating `T` and `U` as distinct types.
//!
//! Which is why this module provides helper [`cast_right()`] and [`cast_left()`]
//! functions:
//!
//! ```rust
//! use ::higher_kinded_types::{ForLt, advanced::type_eq::{self, Is}};
//!
//! fn f<'s, T : ForLt>(a: &'s str)
//! where
//!     T::Of<'s> : Is<EqTo = &'s str>,
//! {
//!     let _: T::Of<'s> = type_eq::cast_left::<T::Of<'s>>(a);
//! }
//! ```
//!
//! But perhaps more interestingly, [`cast_right()`] and [`cast_left()`] are
//! unable to handle _types depending on `T`_, such as `Vec<T>` _vs._ `Vec<U>`.
//!
//! That's when we'd like to use "generic-_over-a-type_ generics", that is,
//! _type_ GATs!
//!
//! …ish: while they're not fully supported (no ergonomic instantiation), it can
//! be intellectually interesting to notice that once we let go of ergonomics,
//! there can be actual usages of type ~~"HKTs"~~ GATs.
//!
//! See, for instance, [`cast_wrapper_right`] and its documentation example.

use crate::advanced::extra_arities::ForTy as ForType;

pub
trait Is {
    type EqTo : ?Sized;
}

impl<T : ?Sized> Is for T {
    type EqTo = Self;
}

/// Given <code>T : [Is]\<EqTo = U\></code>, it allows safely converting any
/// value of type `T` into a value of type `U`.
pub
fn cast_right<T>(it: T)
  -> <T as Is>::EqTo
{
    // For those reading this code and curious about what this function even
    // does (it seems to be a noöp!), it's actually quite subtle:
    //
    //  1. In a non-`: Is`-bounded scenario, Rust knows about the blanket
    //     `T : Is<EqTo = T>` property, and so lets these things type check.
    //
    //     This function body is just one example of it.
    //
    //     These scenarios can be spotted by the need to provide the `as Is>`
    //     when querying the associated type.
    //
    //  2. In a `T : Is<EqTo = U>`-bounded scenario, Rust "forgets" about the
    //     outer blanket impl, which is why it will not type-unify these two
    //     things; on the other hand, it still knows of this generic `cast_right`
    //     function, where it can replace the parameters with the properties
    //     it knows. It thus becomes: `fn cast_right<T>(it: T) -> U;` at the
    //     call-site (remember, the body is type-checked *here*, not at
    //     call-sites), and things Just Work™.
    //
    //     Again, this other scenario can be identified by the fact we can just
    //     write `T::EqTo` (to mean `U`), without needing the `as Is>`
    //     disambiguation 💡
    it
}

/// Like [`cast_right()`], but from `T::EqTo` to `T` this time.
///
/// Given <code>T : [Is]\<EqTo = U\></code>, it allows safely converting any
/// value of type `U` into a value of type `T`.
pub
fn cast_left<T>(it: <T as Is>::EqTo)
  -> T
{
    it
}

/// Given <code>T : [Is]\<EqTo = U\></code>, it allows safely converting any
/// value of type <code>Wrapper[::Of]\<T\></code> into a value of type
/// `Wrapper::Of<U>`.
///
/// [::Of]: ForType
///
/// ```rust
/// use ::higher_kinded_types::advanced::{
///     extra_arities::{For, new_For_type},
///     type_eq::{cast_wrapper_right, Is},
/// };
///
/// fn demo<T : Is<EqTo = u32>>(
///     v: Vec<T>,
/// ) -> Vec<u32>
/// {
///     new_For_type! {
///         type Vec_ = For!(<T> = Vec<T>);
///     }
///
///     cast_wrapper_right::<Vec_, T>(v)
/// }
/// ```
pub
fn cast_wrapper_right<Wrapper: ForType, T>(
    it: Wrapper::Of<T>,
) -> Wrapper::Of<<T as Is>::EqTo>
{
    it
}

/// Like [`cast_wrapper_right()`], but from `T::EqTo` to `T` this time.
pub
fn cast_wrapper_left<Wrapper: ForType, T>(
    it: Wrapper::Of<<T as Is>::EqTo>,
) -> Wrapper::Of<T>
{
    it
}
