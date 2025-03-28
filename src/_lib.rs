//! [ForLifetime]: trait@ForLifetime
#![doc = include_str!("../README.md")]
#![no_std]
#![forbid(unsafe_code)]
#![allow(type_alias_bounds, uncommon_codepoints, unused_braces)]
#![allow(
    // in case `crate::ForLt!` does not resolve, we have the `crate::hkt_macro::*` fallback.
    macro_expanded_macro_exports_accessed_by_absolute_paths,
)]
#![cfg_attr(feature = "better-docs",
    feature(decl_macro, doc_cfg, trait_alias),
)]
#![cfg_attr(feature = "fn_traits",
    feature(unboxed_closures),
)]

#[macro_use]
extern crate macro_rules_attribute;

/// The crate's prelude.
pub
mod prelude {
    #[doc(no_inline)]
    pub use crate::{
        advanced::ForLifetimeMaybeUnsized,
        ForLt,
        ForLifetime,
    };
}

// macro internals
#[doc(hidden)] /** Not part of the public API */ pub
mod ඞ {
    pub use {
        ::core, // or `std`
        crate::{
            advanced::{
                extra_arities::{
                    for_lt_and_lt::WithLifetimes,
                },
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
    struct ForLt<T : ?Sized>(
        ::core::marker::PhantomData<fn(&()) -> &T>,
        ǃ,
    );

    /// Do not use this type!
    pub
    struct ForLtAndLt<T : ?Sized>(
        ::core::marker::PhantomData<fn(&()) -> &T>,
        ǃ,
    );

    use ::never_say_never::Never as ǃ;
}


#[cfg_attr(feature = "docs-rs",
    doc(cfg(advanced)),
)]
pub
mod advanced;

#[cfg(feature = "fn_traits")]
mod fn_traits;

#[allow(unused_imports)]
#[doc(hidden)]
pub use hkt_macro::*;
mod hkt_macro;

mod hkt_muncher;

mod utils;

/// The main trait of the crate. The one expressing `: <'_>`-genericity.
///
/// It is expected to be used as a bound on a `<T>` generic parameter,
/// thereby resulting in a <code><T : [ForLt]></code> generic API, which,
/// conceptually / sort to speak, is to be read as `<T : <'_>>`.
///
/// That is, **a _generic API_ whose generic parameter is, in and of itself,
/// _generic too_!**
///
///   - Such a "generic-generic API" is dubbed _higher-kinded_, which makes a
///     type such as `struct Example<T: <'_>>` then be dubbed _higher-kinded
///     type_, or **HKT**, for short.
///
///     From there, the whole concept of expressing genericity over
///     `: <'_>`-generic types can also be associated with the idea and concept
///     of higher-kinded types, much like the name of this crate indicates.
///
///     So, using this HKT terminology for something other than a type taking
///     a [`: For`-bounded] generic type is, if we are to be pedantic[^haskell]
///     about the topic, an abuse of terminology (one which I'll probably make
///     throughout this documentation).
///
///
/// [^haskell]: For Haskell enthusiasts, this [`: For`-bounded]-ness could be
/// called "Arrow-Kinded", as in, it matches the `… -> *` kind. \
/// \
/// Then, an Arrow-Kinded type which has, inside the `…`, yet another
/// Arrow-Kinded type, is what is called a Higher-Kinded Type: \
/// \
///   - "Arrow-Kinded Type": `… -> *`, such as `ForLt!(<'a> = &'a str) : ForLt`.
///   - Higher-Kinded Type: `(… -> *) -> *`, such as `struct Example<T : ForLt>`.
///
/// [`: For`-bounded]: advanced::extra_arities
///
/// [ForLt]: trait@ForLt
/// [`ForLt`]: trait@ForLt
///
/// It cannot be manually implemented: the only types implementing this trait
/// are the ones produced by the [`ForLt!`] macro.
///
/// ## HKT Usage
///
///  1. Make your API take a generic <code>\<T : [ForLifetime]\></code>
///     parameter (conceptually, a <code>\<T : [Ofᐸᑊ_ᐳ]\></code> parameter).
///
///     Congratulations, you now have a _higher-kinded_ API: your API is
///     not only generic, but it is also taking a parameter which is, in turn,
///     generic.
///
///  1. #### Callers
///
///     Call sites use the [`ForLt!`] macro to produce a type which they
///     can _and must_ turbofish to such APIs. For instance:
///
///       - <code>[ForLt!]\(&str\)</code> for the pervasive reference case
///         (which could also use the <code>[ForRef]\<str\></code> type alias
///         to avoid the macro),
///
///         or <code>[ForLt!]\(Cow\<\'_, str\>\)</code> for more complex
///         lifetime-infected types;
///
///       - <code>[ForLt!]\(u8\)</code> or other owned types work too: it is not
///         mandatory, at the call-site, to be lifetime-infected, it is just
///         _possible_ (maximally flexible API). See [`ForFixed`].
///
///  1. #### Callee/API author
///
///     Make use of this nested genericity in your API!
///
///     Feed, somewhere, a lifetime parameter to this `T`:
///
///     ```rust
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
///          use ::higher_kinded_types::ForLifetime;
///
///          struct Example<'a, 'b, T : ForLifetime> {
///              a: T::Of<'a>,
///              b: T::Of<'b>,
///          }
///          ```
///
///       - wanting to "feed a lifetime later" / to feed a
///         `for<>`-quantified lifetime to your <code>impl [ForLt]</code> type:
///
///          ```rust
///          # #[cfg(any())] macro_rules! ignore {
///          use ::higher_kinded_types::ForLifetime as Ofᐸᑊ_ᐳ; // hopefully illustrative renaming.
///
///          fn slice_sort_by_key<Item, Key : Ofᐸᑊ_ᐳ> (
///              items: &'_ mut [Item],
///              mut get_key: impl FnMut(&'_ Item) -> Key::Of<'_>,
///          )
///          # }
///          ```
///
///          Full example:
///
///          <details class="custom"><summary><span class="summary-box"><span>Click to show</span></span></summary>
///
///          ```rust
///          use ::higher_kinded_types::ForLt;
///
///          fn slice_sort_by_key<Item, Key : ForLt> (
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
///          slice_sort_by_key::<_, ForLt!(&str)>(clients, |c| &c.key); // ✅
///
///          // Important: owned case works too!
///          slice_sort_by_key::<_, ForLt!(u8)>(clients, |c| c.version); // ✅
///
///          # #[cfg(any())] {
///          // But the classic `sort_by_key` stdlib API fails, since it does not use HKTs:
///          clients.sort_by_key(|c| &c.key); // ❌ Error: cannot infer an appropriate lifetime for autoref due to conflicting requirements
///          # }
///          ```
///
///          </details>
///
/// ### Wait a moment; this is just a GAT! Why are you talking of HKTs?
///
/// Indeed, the definition of the <code>[ForLt]</code> trait is basically that
/// of a trait featuring the simplest possible GAT:
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
/// Yes, the `: <'_>` signature pattern of HKTs, and GATs, from this point of
/// view, are quite interchangeable:
///
///   - this whole crate is a demonstration of featuring `: <'_>` HKT idioms
///     through a [`ForLt`] GAT trait (+ some extra `for<>`-quantifications);
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
///       - Real code:
///
///         <details class="custom"><summary><span class="summary-box"><span>Click to show</span></span></summary>
///
///         ```rust
///         use ::higher_kinded_types::ForLt;
///
///         trait LendingIterator {
///             /// Look ma, "no" GATs!
///             type Item: ForLt;
///
///             fn next(&mut self) -> <Self::Item as ForLt>::Of<'_>;
///         }
///         ```
///
///     </details>
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
///     using [`iter::from_fn()`][::core::iter::from_fn]
///   - **[`Future`]**: from
///     <code>FnMut\(\&mut [Context]\<\'_\>\) -\> [Poll]\<Output\></code>
///     using [`future::poll_fn()`][::core::future::poll_fn];
///   - **[`Stream`]**: from
///     `FnMut(Acc) -> impl Future<Output = (Item, Acc)>`
///     using [`stream::unfold()`]
///
/// [`Future`]: ::core::future::Future
/// [Context]: ::core::task::Context
/// [Poll]: ::core::task::Poll
/// [`Stream`]: https://docs.rs/futures/^0.3.28/futures/stream/trait.Stream.html
/// [`stream::unfold()`]: https://docs.rs/futures/^0.3.28/futures/stream/fn.unfold.html
///
/// And that same difference applies to arbitrary GATs _vs._ [`ForLt`]: the
/// ability to produce _ad-hoc_ / on-demand <code>impl [ForLt]</code> types /
/// [`ForLt`] type "expressions", thanks to the [`ForLt!`] macro, is what makes
/// [`ForLt`] convenient and flexible, _vs._ the overly cumbersome aspect of
/// manually using custom GATs.
///
/// Indeed, compare:
///
/// ```rust
/// trait ForLt {
///     type Assoc<'lt>;
/// }
///
/// enum StrRef {}
///
/// impl ForLt for StrRef {
///     type Assoc<'lt> = &'lt str;
/// }
/// ```
///
/// to:
///
/// ```rust
/// # use ::higher_kinded_types::ForLt;
/// type StrRef = ForLt!(<'lt> = &'lt str);
/// ```
///
/// ### Conclusion
///
/// So, to summarize, this <code>[ForLt] = ": \<\'_\>"</code> HKT pattern is just:
///
///   - some GAT API having been _canonical_-ized,
///
///       - much like how, in the realm of closures, the `Fn(Args…) -> R` was
///         picked (_vs._ any other signature-equivalent
///         `Closure<Args, Ret = R>` trait);
///
///   - which can be "inhabited" _on demand_ / in an _ad-hoc_ fashion thanks to
///     the <code>[ForLt!]\(\<\'input\> = Output…\)</code> macro,
///
///       - much like how, in the realm of closures, it is done with the
///         `|input…| output…` closure expressions.
///
/// In other words:
///
/// > `: <'_>` and HKTs are to GATs what closures are to traits.
///
/// (it's the `Fn(Lifetime) -> Type` of the type realm).
///
/// ___
///
/// Finally, another observation which I find interesting, is that:
///
/// ```rust
/// # use ::higher_kinded_types::ForLt;
/// #
/// type A = ForLt!(<'r> = &'r str);
/// // vs.
/// type B        <'r> = &'r str;
/// ```
///
/// is an annoying limitation of Rust, which happens to feature a similar
/// distinction that certain past languages have had between values, and
/// functions, wherein they were treated separately (rather than as first-class
/// citizens, _i.e._, like the other values).
///
/// In Rust, `type B<'r> = &'r str;` suffers from this same kind of limitation,
/// only in the type realm this time: `type B<'r> =` is a special construct,
/// which yields a _"type" constructor_. That is, it yields some syntax, `B`, to
/// which we can feed a lifetime `'lt`, by writing `B<'lt>`, so as to end up
/// with a _type_.
///
/// **But `B`, in and of itself, _is not a type_**, even if we often call it a
/// "generic type" by abuse of terminology.
///
/// Which is why it cannot be fed, _alone_, to some type-generic API that would
/// want to be the one feeding the lifetime parameter: it does not play well
/// with "generic generics"!
///
/// In this example, the only true "generic _type_", that is, the _type_ which
/// is, itself, lifetime-generic, is `A`.
///
/// This is where [`ForLt!`] and HKTs, thus, shine.
pub
trait ForLifetime : seal::WithLifetimeForAny {
    /// "Instantiate lifetime" / "apply/feed lifetime" operation:
    ///
    ///   - Given <code>\<T : [ForLt]\></code>,
    ///
    ///     `T::Of<'lt>` stands for the HKT-conceptual `T<'lt>` type.
    ///
    /// [ForLt]: trait@ForLt
    type Of<'lt>;
}

mod seal {
    pub trait WithLifetimeForAny {}
    impl<T : ?Sized + for<'any> crate::advanced::WithLifetime<'any>> WithLifetimeForAny for T {}
}

/// Shorthand alias.
#[doc(no_inline)]
pub use ForLifetime as ForLt;

crate::utils::cfg_match! {
    feature = "better-docs" => (
        /// <code>: [Ofᐸᑊ_ᐳ]</code> is a hopefully illustrative syntax that
        /// serves as an alias for <code>: [ForLt]</code>.
        ///
        /// [ForLt]: trait@ForLt
        ///
        /// When trying to teach the notion of a HKT / "generic generic API" to
        /// somebody who has never run into it, _e.g._, in introductory
        /// documentation, blog posts, _etc._, the <code>: [Ofᐸᑊ_ᐳ]</code>
        /// syntax ought to be more _intuitive_:
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
        ///   - (the idea being that `: Ofᐸᑊ_ᐳ` looks quite a bit like `: Of<'_>`).
        ///
        ///   - ⚠️ real code should nonetheless be using the <code>: [ForLt]</code>
        ///     syntax: ASCII characters are easier to type with a standard
        ///     keyboard layout, contrary to `Ofᐸᑊ_ᐳ`, which will probably require
        ///     copy-pasting.
        #[doc(cfg(educational))]
        pub trait Ofᐸᑊ_ᐳ = ForLt;
    );

    _ => (
        mod r#trait {
            #![allow(unused)]
            pub use super::*;
            macro_rules! __ {() => ()}
            use __ as ForLt;
        }

        pub trait Ofᐸᑊ_ᐳ where Self : ForLt {}
        impl<T : ?Sized> Ofᐸᑊ_ᐳ for T where Self : ForLt {}
    );
}

/// <code>[ForFixed]\<T\></code> is a macro-free alias for
/// <code>[ForLt!]\(\<\'_unused\> = T\)</code>.
///
/// To be used when the generic lifetime parameter is to be ignored, while
/// calling into some HKT API.
pub
type ForFixed<T : Sized> = ForLt!(T);

/// <code>[ForRef]\<T\></code> is a macro-free alias for
/// <code>[ForLt!]\(\<\'any\> = \&\'any T\)</code>.
pub
type ForRef<T : ?Sized> = ForLt!(&'_ T);

/// <code>[ForRefMut]\<T\></code> is a macro-free alias for
/// <code>[ForLt!]\(\<\'any\> = \&\'any mut T\)</code>.
pub
type ForRefMut<T : ?Sized> = ForLt!(&'_ mut T);

#[cfg(feature = "ui-tests")]
#[doc = include_str!("compile_fail_tests.md")]
mod _compile_fail_tests {}
