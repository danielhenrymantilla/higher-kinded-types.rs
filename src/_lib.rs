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
                Input as r#for,
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

/// The main trait of the crate. The one expressing the `: <'_>`-genericity of a
/// itself generic type ("generic generic").
///
/// [HKT]: trait@HKT
///
/// It is not to be manually implemented: the only types implementing this trait
/// are the ones produced by the [`HKT!`] macro.
///
/// # Usage
///
///  1. Make your API take a generic `<T : HKT>` parameter.
///
///     Congratulations, you now[^1] have a _higher-kinded_ API: your API is
///     not only generic, but it is also taking a parameter which is, in and of
///     itself, generic.
///
///  1. Callers: call sites use the [`HKT!`] macro to produce a type which they
///     can _and must_ turbofish to such APIs. For instance:
///
///       - <code>[HKT!]\(&str\)</code> for the pervasive reference case
///         (which could also use <code>[HktRef]\<str\></code> to avoid the
///         macro),
///
///         or <code>[HKT!]\(Cow\<\'_, str\>\)</code> for more complex
///         lifetime-infected types;
///
///       - <code>[HKT!]\(u8\)</code> or other owned types work too: it is not
///         mandatory, at the call-site, to be lifetime-infected, it is just
///         _possible_ (maximally flexible API).
///
///  1. Callee/API author: make use of this nested genericity in your API!
///
///     Feed, somewhere, a lifetime parameter to this `T`:
///
///     ```rs
///     # #[cfg(any())] macro_rules! ignore {
///     T::__<'some_lifetime_param>
///     # }
///     ```
///
///     There are two situations where this is handy:
///
///       - wanting to feed two different lifetimes to `T`:
///
///          ```rust
///          use ::higher_kinded_types::prelude::*;
///
///          struct Example<'a, 'b, T : HKT> {
///              a: T::__<'a>,
///              b: T::__<'b>,
///          }
///          ```
///
///       - wanting to "feed a lifetime later" / to feed a
///         `for<>`-quantified lifetime to your <code>impl [HKT]</code> type:
///
///          ```rust
///          # #[cfg(any())] macro_rules! ignore {
///          use ::higher_kinded_types::{prelude::*, ᐸᑊ_ᐳ};
///
///          fn slice_sort_by_key<Item, Key : ᐸᑊ_ᐳ> (
///              items: &'_ mut [Item],
///              mut get_key: impl for<'it> FnMut(&'it Item) -> Key::__<'it>,
///          )
///          # }
///          ```
///
///          Full example:
///
///          <details class="custom"><summary><span class="summary-box"><span>Click to show</span></span></summary>
///
///          ```rust
///          use ::higher_kinded_types::prelude::*;
///
///          fn slice_sort_by_key<Item, Key : HKT> (
///              items: &'_ mut [Item],
///              mut get_key: impl for<'it> FnMut(&'it Item) -> Key::__<'it>,
///          )
///          where
///              for<'it> Key::__<'it> : Ord,
///          {
///              items.sort_by(|a: &'_ Item, b: &'_ Item| <Key::__<'_>>::cmp(
///                  &get_key(a),
///                  &get_key(b),
///              ))
///          }
///
///          // Demo:
///          let clients: &mut [Client] = // …;
///          # &mut []; struct Client { key: String, version: u8 }
///
///          slice_sort_by_key::<_, HKT!(&str)>(clients, |c| &c.key); // ✅
///
///          // Important: owned case works too!
///          slice_sort_by_key::<_, HKT!(u8)>(clients, |c| c.version); // ✅
///
///          # #[cfg(any())] {
///          // But the classic `sort_by_key` stdlib API fails, since it does not use HKTs:
///          clients.sort_by_key(|c| &c.key); // ❌ Error: cannot infer an appropriate lifetime for autoref due to conflicting requirements
///          # }
///          ```
///
///          </details>
///
/// [^1]: The bound `T : HKT` is kind of an abuse of terminology: `T` itself is
/// not higher-kinded, the generic API taking `<T : HKT>` is, if we want to be
/// pedantic.
pub
trait HKT : Send + Sync + Unpin + seal::Sealed
// where
//     Self : for<'any> WithLifetime<'any>,
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

mod seal {
    pub trait Sealed {}
    impl<T> Sealed for crate::ඞ::PhantomData<T> {}
}

// impl seal::Sealed for
#[doc(hidden)]
impl<T : ?Sized> HKT for T
where
    Self : for<'any> WithLifetime<'any> + seal::Sealed,
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
        /// somebody who has never run into it, _e.g._, in introductory
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
