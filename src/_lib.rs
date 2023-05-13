#![doc = include_str!("../README.md")]
#![no_std]
#![forbid(unsafe_code)]
#![allow(type_alias_bounds, uncommon_codepoints)]
#![allow(
    // in case `crate::HKT!` does not resolve, we have the `crate::hkt_macro::*` fallback.
    macro_expanded_macro_exports_accessed_by_absolute_paths,
)]
#![cfg_attr(feature = "better-docs",
    feature(decl_macro, rustc_attrs, trait_alias),
)]
#![cfg_attr(feature = "fn_traits",
    feature(unboxed_closures),
)]

/// The crate's prelude.
pub
mod prelude {
    #[doc(no_inline)]
    pub use crate::{
        HKT,
        HktRef,
        HktMut,
    };
}

// macro internals
#[doc(hidden)] /** Not part of the public API */ pub
mod ඞ {
    pub use {
        ::core, // or `std`
        crate::{
            with_lifetime::{
                WithLifetime,
            },
        },
    };
    #[cfg(feature = "fn_traits")]
    pub use {
        crate::{
            fn_traits::{
                Input as For,
                Input as __,
            },
        },
    };

    /// Do not use this type!
    pub
    struct PhantomData<T : ?Sized>(
        ::core::marker::PhantomData<T>,
    );
}

use {
    crate::{
        with_lifetime::WithLifetime,
    },
};

#[cfg(feature = "fn_traits")]
mod fn_traits;

#[allow(unused_imports)]
#[doc(hidden)]
pub use hkt_macro::*;
mod hkt_macro;

mod hkt_muncher;

mod utils;

mod with_lifetime {
    pub
    trait WithLifetime<'lt> : Send + Sync + Unpin {
        type T;
    }

    impl<'lt, T : ?Sized + WithLifetime<'lt>>
        WithLifetime<'lt>
    for
        crate::ඞ::PhantomData<T>
    {
        type T = T::T;
    }
}

pub
trait HKT
where
    Self : for<'any> WithLifetime<'any>,
{
    /// "Instantiate lifetime" / "apply/feed lifetime" operation:
    ///   - Given <code>\<T : [HKT]\></code>,
    ///
    ///     `T::__<'lt>` stands for the conceptual `T<'lt>` type.
    ///
    ///   - Pseudo-code syntax:
    ///     <details      class="custom"><summary><span class="summary-box"><span>Click to hide</span></span></summary>
    ///
    ///     given <code>\<T : [ᐸᑊ_ᐳ]\></code>,
    ///
    ///     `T::__<'lt>` stands for the conceptual `T<'lt>` type.
    ///     </details>
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

crate::utils::cfg_match! {
    feature = "better-docs" => (
        /// <code>: [ᐸᑊ_ᐳ]</code> is a hopefully illustrative syntax that
        /// serves as an alias for <code>: [HKT]</code>.
        ///
        /// [HKT]: trait@HKT
        ///
        /// When trying to teach the notion of a HKT / "generic generic" to
        /// somebody that has never run into it, _e.g._, in introductory
        /// documentation, blog posts, _etc._, the <code>: [ᐸᑊ_ᐳ]</code>
        /// syntax ought to be more _intuitive_:
        ///
        ///   - (the idea being that `: ᐸᑊ_ᐳ` looks quite a bit like `: <'_>`).
        ///
        /// ```rust
        /// use ::higher_kinded_types::*;
        ///
        /// struct Example<'a, 'b, T : ᐸᑊ_ᐳ> {
        ///     a: T::__<'a>,
        ///     b: T::__<'b>,
        /// }
        /// ```
        ///
        ///   - ⚠️ real code should nonetheless be using the <code>: [HKT]</code>
        ///     syntax: ASCII characters are easier to type with a standard
        ///     keyboard layout, contrary to `ᐸᑊ_ᐳ`, which will probably require
        ///     copy-pasting.
        pub trait ᐸᑊ_ᐳ = HKT;
    );

    _ => (
        mod r#trait {
            #![allow(unused)]
            pub use super::*;
            macro_rules! __ {() => ()}
            use __ as HKT;
        }

        pub use r#trait::HKT as ᐸᑊ_ᐳ;
    );
}

/// Shorthand alias for <code>[HKT!]\(\<\'any\> = \&\'any T\)</code>.
pub
type HktRef<T : ?Sized> = HKT!(<'any> = &'any T);

/// Shorthand alias for <code>[HKT!]\(\<\'any\> = \&\'any mut T\)</code>.
pub
type HktMut<T : ?Sized> = HKT!(&'_ mut T);

#[cfg(feature = "ui-tests")]
#[doc = include_str!("compile_fail_tests.md")]
mod _compile_fail_tests {}
