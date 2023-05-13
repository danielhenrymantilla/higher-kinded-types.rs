#![doc = include_str!("../README.md")]
#![no_std]
#![forbid(unsafe_code)]
#![allow(uncommon_codepoints)]
#![cfg_attr(feature = "better-docs", feature(decl_macro, rustc_attrs, trait_alias))]

#[cfg(COMMENTED_OUT)] // <- Remove this when used!
/// The crate's prelude.
pub
mod prelude {
    // …
}

// macro internals
#[doc(hidden)] /** Not part of the public API */ pub
mod ඞ {
    pub use ::core; // or `std`
}

#[doc = include_str!("compile_fail_tests.md")]
mod _compile_fail_tests {}

use {
    ::core::{
        ops::Not as _,
    },
};

#[macro_use]
extern crate macro_rules_attribute;

use with_lifetime::WithLifetime;

mod utils;

#[doc(hidden)] /** Not part of the publidc API */
pub
mod with_lifetime {
    pub
    trait WithLifetime<'lt> : Send + Sync + Unpin {
        type T;
    }

    impl<'lt, T : ?Sized + WithLifetime<'lt>>
        WithLifetime<'lt>
    for
        ::core::marker::PhantomData<T>
    {
        type T = T::T;
    }

    // /// <code>[Feed]\<\'lt, T\></code> is sugar for
    // /// <code>\<T as [WithLifetime]\<\'lt\>::T</code>
    // #[allow(type_alias_bounds)]
    // pub
    // type Feed<'lt, T : ?Sized + WithLifetime<'lt>> = T::T;
}

pub
trait HKT
where
    Self : for<'any> WithLifetime<'any>,
{
    /// "Instantiate lifetime" / "apply/feed lifetime" operation:
    ///   - given a <code>\<T : [HKT]\></code>,
    ///     `T::__<'lt>` will represent, conceptually, the `T<'lt>` type.
    ///   - given a <code>\<T : [ᐸᛌ_ᐳ]\></code>,
    ///     `T::__<'lt>` will represent, conceptually, the `T<'lt>` type.
    ///
    /// [HKT]: trait@HKT
    type __<'lt>;
}

impl<T : ?Sized> HKT for T
where
    Self : for<'any> WithLifetime<'any>,
{
    type __<'lt> = <Self as WithLifetime<'lt>>::T;
}

#[macro_export]
macro_rules! HKT {
    (
        <$lt:lifetime> = $T:ty $(,)?
    ) => (
        $crate::with_lifetime::HKT<
            dyn for<$lt> $crate::with_lifetime::WithLifetime<$lt, T = $T>,
        >
    );

    (
        $T:ty $(,)?
    ) => (
        $crate::HKT!(<'__> = $T)
    );
}

crate::utils::cfg_match! {
    feature = "better-docs" => (
        /// Not necessarily intended for real code, just for code snippets in
        /// documentation, blog posts that will try to mimic language syntax using
        /// unicode look-alikes, to hopefully more easily share the useful intuition
        /// of HKT / "generic generics" semantics.
        ///
        /// ```rust
        /// use ::higher_kinded_types::*;
        ///
        /// struct Example<'a, 'b, T : ᐸᛌ_ᐳ> {
        ///     a: T::__<'a>,
        ///     b: T::__<'b>,
        /// }
        /// ```
        pub trait ᐸᛌ_ᐳ = HKT;
    );

    _ => (
        mod r#trait {
            pub use super::*;
            macro_rules! __ {() => ()}
            use __ as HKT;
        }

        pub use r#trait::HKT as ᐸᛌ_ᐳ;
    );
}
