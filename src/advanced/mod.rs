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
/// [ForLt]: trait@ForLt
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
/// # What for?
///
/// You may wonder why is this trait exposed, what usage can a downstream user possibly make of it.
///
/// The answer lies in an important aspect of the implementation of [`ForLt!`]:
///
/// > **[`ForLt!()`] types implement <code>for\<\'any\> [WithLifetime]\<\'any, …\></code>**
///
/// This in turn, means that it is possible **for downstream users to define their own flavors of
/// the [`ForLifetime`] traits, _with their own custom trait bounds on the end type_**.
///
/// ## Example
///
/// Defining a `ForLifetimeDebug` trait:
///
/// ```rust
/// use ::core::fmt::Debug;
/// use ::higher_kinded_types::{ForLt, advanced::WithLifetime};
///
/// pub trait ForLifetimeDebug {
///     /// You may name this associated type as you like, although consistency with the other
///     /// `ForLifetime` traits make the `Of` naming strongly advisable.
///     type Of<'lt> : Debug + Sized;
///     //                   ^^^^^^^
///     //                   Note: `WithLifetime` defaults to a `?Sized` "output" `Of` type.
///     //                   So it is advisable to be explicit about `Sized` vs. `?Sized`.
/// }
///
/// //             satisfied by every `ForLt!()` invocation
/// //                vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv
/// impl<T : ?Sized + for<'any> WithLifetime<'any, Of : Debug + Sized>>
///     ForLifetimeDebug
/// for
///     T
/// {
///     type Of<'lt> = <T as WithLifetime<'lt>>::Of;
/// }
///
/// // we have now defined a
/// // `trait alias ForLifetimeDebug = for<'any> WithLifetime<'any, Of : Debug + Sized>`.
/// // Note that this ought to be equivalent to having defined a
/// // `trait alias ForLifetimeDebug = for<'any> ForLifetimeMaybeUnsized<Of<'any> : Debug + Sized>`
/// // i.e.,
/// // `trait alias ForLifetimeDebug = for<'any> ForLifetime<Of<'any> : Debug>`
/// // i.e.,
/// // `trait alias ForLifetimeDebug = for<'any> ForLt<Of<'any> : Debug>`
/// //
/// // Maybe having targeted these other traits could have worked just as fine, but Rust is
/// // notorious for eventually running into bugs and limitations when mixing
/// // higher-ranked/`for<>`-quantified trait bounds and GATs together, so it may be more robust
/// // and thus advisable to go back to the "source nougat" of `WithLifetime` for these new
/// // definitions, we never know.
///
/// /// Example usage
/// fn example<T>(
///     f: impl Fn(&str) -> T::Of<'_>,
/// ) -> String
/// where
///     T : ForLifetimeDebug,   /*
///     // above ought to be equivalent to having inlined a:
///     T : for<'any> ForLt<Of<'any> : Debug>,
///     // kind of bound.       */
/// {
///     let local = String::from("Hello, there");
///     let value: T::Of<'_> = f(&*local);
///     format!("{value:?}")
/// }
///
/// assert_eq!(
///     example::<ForLt!(Option<&str>)>(|s| s.get(..4)),
///     format!("{:?}", Some("Hell")),
/// );
/// ```
///
///   - Although it looks like a similar snippet having used
///     <code>for\<\'any\> [ForLt]\<[Of][ForLt::Of]\<\'any\> : [Debug]\></code>
///     for the "`trait alias` definition" would have behaved exactly the same
///     as this `[WithLifetime]`-using snippet…
///
///     Maybe there is no point in exposing this trait after all? 😅
///
/// # Another usage: forgoing / circumventing / bypassing / DIYing the [`ForLt!`] macro
///
/// It could be useful for those loath to use a "magic macro" such as [`ForLt!`], so that they
/// could instead spell out the full `dyn for<'any> WithLifetime<'any, Of = …>` type (hopefully
/// wrapped in a `Sized` newtype to avoid requiring `<T : ForLt>` APIs to add a `T : ?Sized`
/// unbound).
///
/// Or maybe they could want to use their own macro of this sort, wherein they could add some extra
/// autotraits:
///
/// ```rust
/// #[macro_export] macro_rules! foo {() => ()}
/// ```
///
/// ```rust
/// #[macro_export] macro_rules! foo {() => ()}
/// ```
///
/// ```rust
/// use ::higher_kinded_types::{ForLt, advanced::WithLifetime};
/// use ::std::panic::UnwindSafe as WhatAJoke;
///
/// macro_rules! MyUnwindSafeForLt {( $T:ty $(,)? ) => (
///   dyn ::std::panic::UnwindSafe
///     + for<'any>
///       ::higher_kinded_types::advanced::WithLifetime<'any, Of = <ForLt!($T) as ForLt>::Of<'any>>
/// )}
///
/// const _: &dyn WhatAJoke = &::core::marker::PhantomData::<
///     MyUnwindSafeForLt!(&str)
/// >;
/// ```
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
    // for 1.76.0 test; TODO: put inline bounds back.
    Self : for<'any> WithLifetime<'any>,// , Of : Sized>,
    for<'any> <Self as WithLifetime<'any>>::Of : Sized,
{
    type Of<'lt> = <Self as WithLifetime<'lt>>::Of;
}

impl<T : ?Sized> ForLifetimeMaybeUnsized for T
where
    Self : for<'any> WithLifetime<'any>,
{
    type Of<'lt> = <Self as WithLifetime<'lt>>::Of;
}

#[cfg(any())]
mod demo {
    extern crate self as higher_kinded_types;
    use ::core::fmt::Debug;
    use higher_kinded_types::ForLt;

    use super::{ForLifetimeMaybeUnsized, WithLifetime};

    trait ForLtDebug {
        type Of<'__> : Debug;
    }
    impl<T : ?Sized + for<'any> ForLt<Of<'any> : Debug>> ForLtDebug for T {
        type Of<'lt> = <T as ForLt>::Of<'lt>;
    }

    fn rust_bug<'s, T>()
    where
        // T : ForLtDebug,
        // T : ForLtDebug<Of<'s> = &'s str>,
        T : for<'any> ForLt<Of<'any> : Debug>,
        T : ForLt<Of<'s> = &'s str>,
        // T : for<'any> WithLifetime<'any, Of : Debug>,
        // T :           WithLifetime<'s, Of = &'s str>,
    {}
}
