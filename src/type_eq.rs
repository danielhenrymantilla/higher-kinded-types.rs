//! Niche and advanced, you can ignore this to begin with.
//! Module for type-equality hacks.
//!
//! In certain scenarios, you may have a `T : HKT` in scope which you want to
//! constrain to match some specific type when fed some specific lifetime;
//! _e.g._
//!
//! ```rust ,ignore
//! //! pseudo-code
//! <'r, T : HKT>
//! where
//!     T::__<'r> = &'r str
//! ```
//!
//! In that case, you can actually just write the corresponding real code:
//!
//! ```rust
//! # use ::higher_kinded_types::HKT;
//! #
//! fn f<'s, T : HKT>(s: &'s str)
//! where
//!     T : HKT<__<'s> = &'s str>,
//! {
//!     let _: T::__<'s> = s;
//! }
//! ```
//!
//! which does work 🙂.
//!
//! It even works with higher-order lifetimes!
//!
//! ```rust
//! # use ::higher_kinded_types::HKT;
//! #
//! fn f<T : HKT>(s: String)
//! where
//!     T : for<'local> HKT<__<'local> = &'local str>,
//! {
//!     let _local: T::__<'_> = &s;
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
//! Which is why this module provides helper [`cast_into()`] and [`cast_from()`]
//! functions:
//!
//! ```rust
//! use ::higher_kinded_types::{HKT, type_eq::{self, Is}};
//!
//! fn f<'a, T : HKT>(a: &'a str)
//! where
//!     T::__<'a> : Is<EqTo = &'a str>,
//! {
//!     let _: T::__<'a> = type_eq::cast_from::<T::__<'a>>(a);
//! }
//! ```
//!
//! But perhaps more interestingly, [`cast_into()`] and [`cast_from()`] are
//! unable to handle _types depending on `T`_, such as `Vec<T>` _vs._ `Vec<U>`.
//!
//! That's when we'd like to use "generic-_over-a-type_ generics", that is,
//! _type_ HKTs!
//!
//! …ish: while they're not fully supported (no ergonomic instantiation), it can
//! be intellectually interesting to notice that once we let go of ergonomics,
//! there can be actual usages of type ~~"HKTs"~~ GATs.
//!
//! See, for instance, the [`TypeGat`] trait and documentation example.

pub
trait Is {
    type EqTo : ?Sized;
}

impl<T : ?Sized> Is for T {
    type EqTo = Self;
}

pub
fn cast_into<T>(it: T)
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
    //     things; on the other hand, it still knows of this generic `cast_into`
    //     function, where it can replace the parameters with the properties
    //     it knows. It thus becomes: `fn cast_into<T>(it: T) -> U;` at the
    //     call-site (remember, the body is type-checked *here*, not at
    //     call-sites), and things Just Work™.
    //
    //     Again, this other scenario can be identified by the fact we can just
    //     write `T::EqTo` (to mean `U`), without needing the `as Is>`
    //     disambiguation 💡
    it
}

pub
fn cast_from<T>(it: <T as Is>::EqTo)
  -> T
{
    it
}

/// ```rust
/// use ::higher_kinded_types::type_eq::{Is, TypeGat};
///
/// fn demo<T : Is<EqTo = u32>>(
///     v: Vec<T>,
/// ) -> Vec<u32>
/// {
///     enum Vec_ {}
///
///     impl TypeGat for Vec_ {
///         type Of<T> = Vec<T>;
///     }
///
///     Vec_::cast_into(v)
/// }
/// ```
pub
trait TypeGat {
    type Of<T>;

    fn cast_into<T>(
        it: Self::Of<T>,
    ) -> Self::Of<<T as Is>::EqTo>
    {
        it
    }


    fn cast_from<T>(
        it: Self::Of<<T as Is>::EqTo>,
    ) -> Self::Of<T>
    {
        it
    }
}
