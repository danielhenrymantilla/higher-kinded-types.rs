//! Niche and advanced, you can ignore this to begin with.
//! Module for other [`ForLt`]-like constructs: extended to various "arities".
//!
//! For instance:
//!   - [`ForTy`] so as to be generic over a type parameter,
//!   - [`ForLtAndLt`] so as to be generic over _two_ lifetime parameters,
//!   - [`ForLtAndTy`] so as to be generic over a lifetime parameter and a type
//!     parameter.
//!
//! [`ForLt`]: trait@ForLt

#[doc(inline)]
pub use crate::ForLifetime as ForLt;

use crate::utils::macro_export;

/// Genericity over a _type_ parameter.
///
/// Note: producing an <code>impl [ForTy]</code> type is not as easy as for
/// an <code>impl [ForLt]</code> type or an <code>impl [ForLtAndLt]</code> type.
///
/// Indeed, these can be done by the [`For!`] macro, internally, thanks to the
/// `for<'lifetimes…>`-quantification that Rust offers.
///
/// Alas, there is no `for<T>`-quantification in Rust (yet?).
///
/// Thence the need to manually define a new type for which to write a dedicated
/// <code>impl [ForTy] for ThatType {</code> item.
///
/// Since this is a bit cumbersome, the [`new_For_type!`] convenience macro is
/// offered to let it take care of the red tape.
///
/// See its documentation for more info.
///
/// [ForLt]: trait@ForLt
pub
trait ForTy : Send + Sync + Unpin {
    type Of<T>;
}

/// Genericity over _two_ lifetime parameters.
///
/// It cannot be manually implemented: the only types implementing this trait
/// are the ones produced by the [`For!`] macro.
///
/// ## Examples
///
/// ```rust
/// use {
///     ::core::{
///         future::{self, Future},
///         task::Context,
///     },
///     ::higher_kinded_types::{
///         extra_arities::*,
///     },
/// };
///
/// /// An `impl ForLtAndLt` yielding `&mut Context<'_>`.
/// type RefMutContext = For!(<'r, 'cx> = &'r mut Context<'cx>);
///
/// async fn demo() {
///     let mut sub_future = Box::pin(sub_future()); // or `pin!`
///     future::poll_fn(|cx: <RefMutContext as ForLtAndLt>::Of<'_, '_>| {
///         sub_future.as_mut().poll(cx)
///     }).await
/// }
///
/// async fn sub_future() {
///     // …
/// }
/// ```
pub
trait ForLtAndLt : for_lt_and_lt::Sealed {
    type Of<'a, 'b>;
}

/// Genericity over a _lifetime_ and a _type_ parameters.
///
/// Note: the same remarks as for [`ForTy`] apply here: see [`new_For_type!`]'s
/// documentation for more info and examples about defining and using such
/// types.
pub
trait ForLtAndTy : Send + Sync + Unpin {
    type Of<'lt, T : 'lt>;
}

/// Same as [`crate::ForLifetime`], but for enforcing covariance of
/// `Self::Of<'_>` (over `'_`).
///
/// ## Example
///
/// ```rust
/// use ::higher_kinded_types::extra_arities::*;
///
/// //                                👇
/// fn higher_kinded_api<'caller, T : CovariantForLt>(
///     caller: T::Of<'caller>,
///     from_str: impl FnOnce(&str) -> T::Of<'_>,
/// ) -> bool
/// where
///     for<'callee>
///         T::Of<'callee> : PartialEq
///     ,
/// {
///     let local: String = ::std::fs::read_to_string(file!()).expect("demo");
///     let callee: T::Of<'_> = from_str(&local);
///     let comparison = {
///         callee == T::covariant_cast(caller) // 👈
///     };
///     comparison
/// }
///
/// new_For_type! {
///     type StrRef = For!(#![covariant]<'r> = &'r str);
/// }
///
/// higher_kinded_api::<StrRef>("…", |s| s);
/// ```
///
/// ### Counter-example
///
/// ```rust ,compile_fail
/// use ::higher_kinded_types::extra_arities::*;
///
/// new_For_type! {
///     type NotCov = For!(#![covariant]<'r> = &'r mut &'r str);
/// }
/// ```
///
/// yields:
///
/// ```rust ,compile_fail
/// # let () = 42; /*
/// error: lifetime may not live long enough
///  --> src/lib.rs:126:1
///   |
/// 6 | / new_For_type! {
/// 7 | |     type NotCov = For!(#![covariant]<'r> = &'r mut &'r str);
/// 8 | | }
///   | | ^
///   | | |
///   | | lifetime `'if_you_are_getting_this_error` defined here
///   | |_lifetime `'it_means_your_type_is_not_covariant` defined here
///   |   associated function was supposed to return data with lifetime `'it_means_your_type_is_not_covariant` but it is returning data with lifetime `'if_you_are_getting_this_error`
///   |
///   = help: consider adding the following bound: `'if_you_are_getting_this_error: 'it_means_your_type_is_not_covariant`
///   = note: requirement occurs because of a mutable reference to `&str`
///   = note: mutable references are invariant over their type parameter
///   = help: see <https://doc.rust-lang.org/nomicon/subtyping.html> for more information about variance
///   = note: this error originates in the macro `$crate::ඞFor` which comes from the expansion of the macro `new_For_type` (in Nightly builds, run with -Z macro-backtrace for more info)
/// # */
/// ```
pub
trait CovariantForLt {
    /// In order to help palliate WF-bounds, this trait carries such a bound.
    type Of<'lt>
    where
        Self : 'lt,
    ;

    /// The actual "proof" which higher-kinded callees dealing with implementors
    /// of this trait can use in order to take advantage of variance.
    fn covariant_cast<'smol, 'humongous : 'smol>(
        it: Self::Of<'humongous>,
    ) -> Self::Of<'smol>
    ;
}

/// Variadic version of [`crate::ForLt!`], suitable for the [`For…` traits of
/// this module][self#traits].
///
/// ### Syntax
///
///   - #### `ForLt`
///
///     ```rust
///     # use ::higher_kinded_types::extra_arities::For;
///     # mod some { pub use ::std::borrow::Cow as Arbitrary; }
///     # use str as Type; let _:
///     For!(<'r> = some::Arbitrary<'r, Type>)
///     # ;
///     ```
///
///       - Notice how there deliberately is no _shorthand syntax_ available
///         (for the sake of readability). Use [`crate::ForLt!`] for that.
///
///   - #### `ForLtAndLt`
///
///     ```rust
///     # use ::higher_kinded_types::extra_arities::For;
///     # mod some { pub type Arbitrary<'a, 'b, T> = &'a &'b T; }
///     # use str as Type; let _:
///     For!(<'a, 'b> = some::Arbitrary<'a, 'b, Type>)
///     # ;
///     ```
///
///   - #### `ForTy`
///
///       - ⚠️ to be used inside a [`new_For_type!`] invocation!
///
///     ```rust
///     # #[cfg(any())] macro_rules! ignore {
///     For!(<T> = Vec<T>)
///     # }
///     ```
///
///   - #### `ForLtAndTy`
///
///       - ⚠️ to be used inside a [`new_For_type!`] invocation!
///
///     ```rust
///     # #[cfg(any())] macro_rules! ignore {
///     For!(<'r, T> = &'r mut T)
///     # }
///     ```
///
///   - #### `CovariantForLt`
///
///       - ⚠️ to be used inside a [`new_For_type!`] invocation!
///
///     ```rust
///     # #[cfg(any())] macro_rules! ignore {
///     For!(#![covariant]<'r> = &'r mut String)
///     # }
///     ```
#[macro_export] #[doc(hidden)]
macro_rules! ඞFor_ {
    (
        $($implementation_details_hidden:tt)*
    ) => (
        $crate::ඞFor! { $($implementation_details_hidden)* }
    );
} #[doc(inline)] pub use ඞFor_ as For;

#[doc(hidden)]
#[macro_export]
macro_rules! ඞFor {
    (
        <$lt:lifetime> = $Type:ty $(,)?
    ) => (
        $crate::ForLt! { <$lt> = $Type }
    );

    (
        <$a:lifetime, $b:lifetime> = $Type:ty $(,)?
    ) => (
        $crate::ඞ::ForLtAndLt<
            dyn for<$a, $b> $crate::ඞ::WithLifetimes<$a, $b, T = $Type>,
        >
    );

    (
        <$($lt:lifetime ,)? $T:ident> = $Type:ty $(,)?
    ) => (
        ::core::compile_error! { concat!("Usage:
new_For_type! {
    /// Attrs…
    pub type TypeName = For!(
        ", stringify!(<$($lt ,)? $T> = $Type), "
    );
}\
        ")}
    );

    (
        #[name($pub:tt $Name:ident)]
        $(#$attr:tt)*
        <$T:ident> = $Type:ty $(,)?
    ) => (
        $(#$attr)*
        $pub
        struct $Name(fn(&()) -> &mut Self);

        impl $crate::extra_arities::ForTy for $Name {
            type Of<$T> = $Type;
        }
    );

    (
        #[name($pub:tt $Name:ident)]
        $(#$attr:tt)*
        <$lt:lifetime, $T:ident> = $Type:ty $(,)?
    ) => (
        $(#$attr)*
        $pub
        struct $Name(fn(&()) -> &mut Self);

        impl $crate::extra_arities::ForLtAndTy for $Name {
            type Of<$lt, $T : $lt> = $Type;
        }
    );

    (
        #[name($pub:tt $Name:ident)]
        $(#[$attr:meta])*
        #![covariant] <$lt:lifetime> = $Type:ty $(,)?
    ) => (
        $(#[$attr])*
        $pub
        struct $Name(fn(&()) -> &mut Self);

        impl $crate::extra_arities::CovariantForLt for $Name {
            type Of<$lt> = $Type
            where
                Self : $lt,
            ;

            // lifetimes renamed for hopefully nicer diagnostics
            #[inline]
            fn covariant_cast<
                'if_you_are_getting_this_error,
                'it_means_your_type_is_not_covariant,
            >(
                it: Self::Of<'it_means_your_type_is_not_covariant>,
            ) -> Self::Of<'if_you_are_getting_this_error>
            where
                Self : 'if_you_are_getting_this_error
                     + 'it_means_your_type_is_not_covariant,
                'it_means_your_type_is_not_covariant
                    : 'if_you_are_getting_this_error,
            {
                it
            }
        }
    );

    (
        #[name($pub:tt $Name:ident)]
        $(#$attr:tt)*
        < $($rest:tt)*
    ) => (
        $(#$attr)*
        $pub type $Name = $crate::extra_arities::For!(< $($rest)* );
    );
}

/// Define (new) `type`s with the desired `For` semantics.
///
/// For technical reasons (lack of `for<T>` quantification), this more
/// roundabout syntax is needed for `For` types involving _type_ genericity.
///
/// ## Examples
///
/// ### `ForTy`
///
/// ```rust
/// use ::higher_kinded_types::extra_arities::*;
///
/// new_For_type! {
///     /// Attributes such as docstrings are obviously allowed.
///     pub type GenericVec = For!(<T> = Vec<T>);
///
///     pub type GenericOption = For!(<T> = Option<T>);
///     pub type GenericBool = For!(<T> = bool);
/// }
///
/// struct HktExample<Collection: ForTy> {
///     ints: Collection::Of<i32>,
///     strings: Collection::Of<String>,
/// }
///
/// let v = HktExample::<GenericVec> {
///     ints: vec![42, 27],
///     strings: vec!["Hello,".into(), "World!".into()],
/// };
///
/// let o = HktExample::<GenericOption> {
///     ints: Some(42),
///     strings: None,
/// };
///
/// let b = HktExample::<GenericBool> {
///     ints: false,
///     strings: true,
/// };
/// ```
///
/// ### `ForLtAndTy`
///
/// ```rust
/// use ::higher_kinded_types::extra_arities::*;
///
/// new_For_type! {
///     type SharedRef = For!(<'r, T> = &'r T);
///     type ExclusiveRef = For!(<'r, T> = &'r mut T);
/// }
///
/// trait IterMaybeMut<Ref: ForLtAndTy> : Sized {
///     type Item;
///
///     fn iter<'r>(this: Ref::Of<'r, Self>)
///       -> Box<dyn 'r + Iterator<Item = Ref::Of<'r, Self::Item>>>
///     where
///         Self : 'r,
///         Self::Item : 'r,
///     ;
/// }
///
/// impl<T> IterMaybeMut<SharedRef> for Vec<T> {
///     type Item = T;
///
///     fn iter<'r>(this: &'r Vec<T>)
///       -> Box<dyn 'r + Iterator<Item = &'r T>>
///     where
///         Self : 'r,
///         Self::Item : 'r,
///     {
///         Box::new(this.into_iter())
///     }
/// }
///
/// impl<T> IterMaybeMut<ExclusiveRef> for Vec<T> {
///     type Item = T;
///
///     fn iter<'r>(this: &'r mut Vec<T>)
///       -> Box<dyn 'r + Iterator<Item = &'r mut T>>
///     where
///         Self : 'r,
///         Self::Item : 'r,
///     {
///         Box::new(this.into_iter())
///     }
/// }
///
///
/// /// This function is a `iter{,_mut}().for_each()` on `Vec<T>` which is
/// /// generic over the `mut`-ness! It handles `&Vec<T>` and `&mut Vec<T>`! 🔥
/// fn vec_for_each<'r, T, Ref : ForLtAndTy>(
///     vec: Ref::Of<'_, Vec<T>>,
///     mut f: impl FnMut(usize, Ref::Of<'_, T>),
/// )
/// where
///     Vec<T> : IterMaybeMut<Ref, Item = T>,
/// {
///     IterMaybeMut::<Ref>::iter(vec)
///         .enumerate()
///         .for_each(|(i, x)| f(i, x))
/// }
///
/// let mut vec = vec![16, 42, 0];
/// vec_for_each::<i32, ExclusiveRef>(&mut vec, |_, x| *x += 27);
/// vec_for_each::<i32, SharedRef>(&vec, |i, &x| {
///     println!("{x}");
///     if i == 1 {
///         assert_eq!(x, 42 + 27);
///     }
/// });
///
/// // Actually, usage of `'r` is not mandatory:
/// new_For_type! {
///     type Owned = For!(<'r, T> = T);
/// }
///
/// impl<T> IterMaybeMut<Owned> for Vec<T> {
///     type Item = T;
///
///     fn iter<'r>(this: Vec<T>)
///       -> Box<dyn 'r + Iterator<Item = T>>
///     where
///         Self : 'r,
///         Self::Item : 'r,
///     {
///         Box::new(this.into_iter())
///     }
/// }
///
/// //                                          owned!
/// //                                          vvv
/// vec_for_each::<i32, Owned>(vec, |_, _owned: i32| {});
/// ```
#[apply(macro_export)]
macro_rules! new_For_type {
    ($(
        $( #$attr:tt )*
        $pub:vis
        type $Name:ident =
            $($(@$if_leading:tt)?
                ::
            )?
            $($macro:ident)::+ ! ( $($args:tt)* )
        ;
    )*) => ($(
        $($($if_leading)? :: )? $($macro)::+ ! {
            #[name($pub $Name)]
            $(#$attr)*
            $($args)*
        }
    )*);
}

pub(crate)
mod for_lt_and_lt {
    pub trait Sealed : Send + Sync + Unpin {}

    pub trait WithLifetimes<'a, 'b> {
        type T;
    }

    impl<T : ?Sized>
        Sealed
    for
        crate::ඞ::ForLtAndLt<T>
    where
        T : for<'a, 'b> WithLifetimes<'a, 'b>,
    {}

    impl<T : ?Sized>
        super::ForLtAndLt
    for
        crate::ඞ::ForLtAndLt<T>
    where
        T : for<'a, 'b> WithLifetimes<'a, 'b>,
    {
        type Of<'a, 'b> = <T as WithLifetimes<'a, 'b>>::T;
    }
}
