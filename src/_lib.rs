//! [Gat]: trait@Gat
#![doc = include_str!("../README.md")]
#![no_std]
#![forbid(unsafe_code)]
#![allow(type_alias_bounds, uncommon_codepoints)]
#![allow(
    // in case `crate::Gat!` does not resolve, we have the `crate::hkt_macro::*` fallback.
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
        Gat,
        GatRef,
        GatMut,
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
    struct Gat<T : ?Sized>(
        ::core::marker::PhantomData<T>,
        ǃ,
    );

    use ::never_say_never::Never as ǃ;
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

pub
mod type_eq;

mod utils;

mod with_lifetime {
    pub
    trait WithLifetime<'lt> : Send + Sync + Unpin {
        type T;
    }

    impl<'lt, T : ?Sized + WithLifetime<'lt>>
        WithLifetime<'lt>
    for
        crate::ඞ::Gat<T>
    {
        type T = T::T;
    }
}

/// The main trait of the crate. The one expressing the `: <'_>`-genericity of
/// an itself generic type ("generic generic" / Higher-Kinded Types).
///
/// [Gat]: trait@Gat
/// [`Gat`]: trait@Gat
///
/// It is not to be manually implemented: the only types implementing this trait
/// are the ones produced by the [`Gat!`] macro.
///
/// ## HKT Usage
///
///  1. Make your API take a generic <code>\<T : [Gat]\></code> parameter
///     (conceptually, a <code>\<T : [Ofᐸᑊ_ᐳ]\></code> parameter).
///
///     Congratulations, you now[^1] have a _higher-kinded_ API: your API is
///     not only generic, but it is also taking a parameter which is, in turn,
///     generic.
///
///  1. #### Callers
///
///     Call sites use the [`Gat!`] macro to produce a type which they
///     can _and must_ turbofish to such APIs. For instance:
///
///       - <code>[Gat!]\(&str\)</code> for the pervasive reference case
///         (which could also use the <code>[GatRef]\<str\></code> type alias to
///         avoid the macro),
///
///         or <code>[Gat!]\(Cow\<\'_, str\>\)</code> for more complex
///         lifetime-infected types;
///
///       - <code>[Gat!]\(u8\)</code> or other owned types work too: it is not
///         mandatory, at the call-site, to be lifetime-infected, it is just
///         _possible_ (maximally flexible API).
///
///  1. #### Callee/API author
///
///     Make use of this nested genericity in your API!
///
///     Feed, somewhere, a lifetime parameter to this `T`:
///
///     ```rs
///     # #[cfg(any())] macro_rules! ignore {
///     T::Of<'some_lifetime_param>
///     # }
///     ```
///
///     There are two situations where this is handy:
///
///       - wanting to feed two different lifetimes to `T`:
///
///          ```rust
///          use ::higher_kinded_types::Gat;
///
///          struct Example<'a, 'b, T : Gat> {
///              a: T::Of<'a>,
///              b: T::Of<'b>,
///          }
///          ```
///
///       - wanting to "feed a lifetime later" / to feed a
///         `for<>`-quantified lifetime to your <code>impl [Gat]</code> type:
///
///          ```rust
///          # #[cfg(any())] macro_rules! ignore {
///          use ::higher_kinded_types::Gat as Ofᐸᑊ_ᐳ;
///
///          fn slice_sort_by_key<Item, Key : Ofᐸᑊ_ᐳ> (
///              items: &'_ mut [Item],
///              mut get_key: impl for<'it> FnMut(&'it Item) -> Key::Of<'it>,
///          )
///          # }
///          ```
///
///          Full example:
///
///          <details class="custom"><summary><span class="summary-box"><span>Click to show</span></span></summary>
///
///          ```rust
///          use ::higher_kinded_types::Gat;
///
///          fn slice_sort_by_key<Item, Key : Gat> (
///              items: &'_ mut [Item],
///              mut get_key: impl for<'it> FnMut(&'it Item) -> Key::Of<'it>,
///          )
///          where
///              for<'it> Key::Of<'it> : Ord,
///          {
///              items.sort_by(|a: &'_ Item, b: &'_ Item| <Key::Of<'_>>::cmp(
///                  &get_key(a),
///                  &get_key(b),
///              ))
///          }
///
///          // Demo:
///          let clients: &mut [Client] = // …;
///          # &mut []; struct Client { key: String, version: u8 }
///
///          slice_sort_by_key::<_, Gat!(&str)>(clients, |c| &c.key); // ✅
///
///          // Important: owned case works too!
///          slice_sort_by_key::<_, Gat!(u8)>(clients, |c| c.version); // ✅
///
///          # #[cfg(any())] {
///          // But the classic `sort_by_key` stdlib API fails, since it does not use HKTs:
///          clients.sort_by_key(|c| &c.key); // ❌ Error: cannot infer an appropriate lifetime for autoref due to conflicting requirements
///          # }
///          ```
///
///          </details>
///
/// [^1]: If we want to be pedantic, the bound `T : Gat` is kind of an abuse of
/// terminology: `T` itself is not higher-kinded, the generic API taking
/// `<T : Gat>` is.
///
/// ### Wait a moment; this is just a GAT! Why are you talking of HKTs?
///
/// Indeed, the definition of the <code>[Gat]</code> trait is basically that of
/// a trait featuring the simplest possible GAT:
///
/// ```rust
/// trait Trait { // basic trait
///     type Assoc<'lt>; // Associated Type which is itself Generic = GAT.
/// }
///
/// struct Struct<'a, 'b, T : Trait> {
///     a: T::Assoc<'a>,
///     b: T::Assoc<'b>,
/// }
/// ```
///
/// Yes, the `: <'_>` signature pattern of HKTs, and GATs, from this point of view,
/// are quite interchangeable:
///
///   - this whole crate is a demonstration of featuring `: <'_>` idioms through
///     a [`Gat`] (+ some extra `for<>`-quantifications);
///
///   - in a world with HKTs and `: <'_>` as a first-class construct, GATs could
///     then just be HKT Associated Types (HATs instead of GATs 🤠).
///
///     ```rust ,ignore
///     //! pseudo-code!
///     trait LendingIterator {
///         type Item: <'_>;
///
///         fn next(&mut self) -> Self::Item<'_>;
///     }
///     ```
///
/// In a way, the similarity between these two paradigms is akin to that of
/// closure _vs._ object in more classic programming: you can always pick some
/// canonical object interface, say:
///
/// ```rust
/// trait Closure<Args> {
///     type Ret;
///
///     fn call(&self, _: Args) -> Self::Ret;
/// }
/// ```
///
/// and then use `Closure<Args, Ret = …>` where we currently use
/// `Fn(Args…) -> Output`: that is, the _canonical_ `Fn…` traits can easily be
/// polyfilled with any arbitrary trait of our choice featuring the same
/// functional API (same method signature).
///
/// or, _vice versa_, never define custom traits or interfaces, and always
/// use closures:
///
/// ```rust ,ignore
/// trait Display = Fn(&mut fmt::Formatter<'_>) -> fmt::Result;
/// // etc.
/// ```
///
///   - The astute reader may notice that we lose the _nominal typing_ aspect
///     of our current traits, which is what lets us, for instance, distinguish
///     between `Display` and `Debug`, even if both traits, _structurally_, are
///     equivalent / have exactly the same function signature.
///
///       - In general, Rust traits go way beyond the sheer API of their
///         methods. They can be used as (sometimes `unsafe`) marker traits, or
///         other API promises, _etc._
///
/// So, closures are just one specific interface/trait shape, which we could use
/// pervasively everywhere, if we did not mind the loss of "nominal typing" (the
/// trait name).
///
/// But they're actually more: closures would not be near as useful as they are
/// if we did not have **closure expressions**!
///
/// In fact, closure expressions are so handy that nowadays we have a bunch of
/// `impl Trait` constructors that take the raw/bare API/signature as a closure,
/// and then wrap it within the "name" of the trait:
///
///   - **[`Iterator`]**: from
///     `FnMut() -> Option<Item>`
///     thanks to [`iter::from_fn()`][::core::iter::from_fn]
///   - **[`Future`]**: from
///     <code>FnMut\(\&mut [Context]\<\'_\>\) -\> [Poll]\<Output\></code>
///     thanks to [`future::poll_fn()`][::core::future::poll_fn];
///   - **[`Stream`]**: from
///     `FnMut(Acc) -> impl Future<Output = (Item, Acc)>`
///     thanks to [`stream::unfold()`]
///
/// [`Future`]: ::core::future::Future
/// [Context]: ::core::task::Context
/// [Poll]: ::core::task::Poll
/// [`Stream`]: https://docs.rs/futures/^0.3.28/futures/stream/trait.Stream.html
/// [`stream::unfold()`]: https://docs.rs/futures/^0.3.28/futures/stream/fn.unfold.html
///
/// And that same difference applies to arbitrary GATs _vs._ [`Gat`]: the ability to
/// produce _ad-hoc_ / on-demand <code>impl [Gat]</code> types / [`Gat`] type
/// "expressions", thanks to the [`Gat!`] macro, is what makes [`Gat`] convenient
/// and flexible, _vs._ the overly cumbersome aspect of manually using custom
/// GATs.
///
/// Indeed, compare:
///
/// ```rust
/// trait Gat {
///     type Assoc<'lt>;
/// }
///
/// enum StrRef {}
///
/// impl Gat for StrRef {
///     type Assoc<'lt> = &'lt str;
/// }
/// ```
///
/// to:
///
/// ```rust
/// # use ::higher_kinded_types::Gat;
/// type StrRef = Gat!(<'lt> = &'lt str);
/// ```
///
/// #### Conclusion
///
/// So, to summarize, this <code>[Gat] = ": \<\'_\>"</code> HKT pattern is just:
///
///   - some GAT API having been _canonical_-ized,
///
///       - much like how, in the realm of closures, the `Fn(Args…) -> R` was
///         picked (_vs._ any other signature-equivalent
///         `Closure<Args, Ret = R>` trait);
///
///   - which can be "inhabited" _on demand_ / in an _ad-hoc_ fashion thanks to
///     the <code>[Gat!]\(\<\'input\> = Output…\)</code> macro,
///
///       - much like how, in the realm of closures, it is done with the
///         `|input…| output…` closure expressions.
///
/// In other words:
///
/// > `: <'_>` and HKTs are to GATs what closures are to traits.
///
///   - (it's the `Fn(Lifetime) -> Type` of the type realm)
///
/// Finally, another observation which I find interesting, is that:
///
/// ```rust
/// # use ::higher_kinded_types::Gat;
/// #
/// type A = Gat!(<'r> = &'r str);
/// // vs.
/// type B        <'r> = &'r str;
/// ```
///
/// is an annoying limitation of Rust, which happens to feature a similar
/// distinction that certain past languages have had between values, and
/// functions, which are treated separately (rather than as first-class citizens
/// / like the other values).
///
/// In Rust, `type B<'r> = &'r str;` suffers from this kind of limitation, only
/// in the type realm: `type B<'r> =` is a special construct, which yields a
/// _"type" constructor_. That is, it yields some syntax, `B`, to which we can
/// feed a lifetime `'lt`, by writing `B<'lt>`, so as to end up with a _type_.
///
/// **But `B`, in and of itself, _is not a type_**, even if we often call it a
/// "generic type" by abuse of terminology.
///
/// Which is why it cannot be fed, _alone_, to some type-generic API that would
/// want to be the one feeding the lifetime parameter: it does not play well
/// with "generic generics"!
///
/// The real "generic type", that is, the _type_, which is, itself,
/// lifetime-generic, in this example, is `A`
///
/// This is where [`Gat!`] and HKTs, thus, shine.
pub
trait Gat : Send + Sync + Unpin + seal::Sealed
// where
//     Self : for<'any> WithLifetime<'any>,
{
    /// "Instantiate lifetime" / "apply/feed lifetime" operation:
    ///   - Given <code>\<T : [Gat]\></code>,
    ///
    ///     `T::Of<'lt>` stands for the HKT-conceptual `T<'lt>` type.
    ///
    /// [Gat]: trait@Gat
    type Of<'lt>;
}

mod seal {
    pub trait Sealed {}
    impl<T : ?Sized> Sealed for crate::ඞ::Gat<T> {}
}

// impl seal::Sealed for
#[doc(hidden)]
impl<T : ?Sized> Gat for T
where
    Self : for<'any> WithLifetime<'any> + seal::Sealed,
{
    type Of<'lt> = <Self as WithLifetime<'lt>>::T;
}

crate::utils::cfg_match! {
    feature = "better-docs" => (
        /// <code>: [Ofᐸᑊ_ᐳ]</code> is a hopefully illustrative syntax that
        /// serves as an alias for <code>: [Gat]</code>.
        ///
        /// [Gat]: trait@Gat
        ///
        /// When trying to teach the notion of a Gat / "generic generic" to
        /// somebody who has never run into it, _e.g._, in introductory
        /// documentation, blog posts, _etc._, the <code>: [Ofᐸᑊ_ᐳ]</code>
        /// syntax ought to be more _intuitive_:
        ///
        ///   - (the idea being that `: Ofᐸᑊ_ᐳ` looks quite a bit like `: Of<'_>`).
        ///
        /// ```rust
        /// use ::higher_kinded_types::*;
        ///
        /// struct Example<'a, 'b, T : Ofᐸᑊ_ᐳ> {
        ///     a: T::Of<'a>,
        ///     b: T::Of<'b>,
        /// }
        /// ```
        ///
        ///   - ⚠️ real code should nonetheless be using the <code>: [Gat]</code>
        ///     syntax: ASCII characters are easier to type with a standard
        ///     keyboard layout, contrary to `Ofᐸᑊ_ᐳ`, which will probably require
        ///     copy-pasting.
        pub trait Ofᐸᑊ_ᐳ = Gat;
    );

    _ => (
        mod r#trait {
            #![allow(unused)]
            pub use super::*;
            macro_rules! __ {() => ()}
            use __ as Gat;
        }

        pub use r#trait::Gat as Ofᐸᑊ_ᐳ;
    );
}

/// Shorthand alias for <code>[Gat!]\(\<\'any\> = \&\'any T\)</code>.
pub
type GatRef<T : ?Sized> = Gat!(&'_ T);

/// Shorthand alias for <code>[Gat!]\(\<\'any\> = \&\'any mut T\)</code>.
pub
type GatMut<T : ?Sized> = Gat!(&'_ mut T);

#[cfg(feature = "ui-tests")]
#[doc = include_str!("compile_fail_tests.md")]
mod _compile_fail_tests {}
