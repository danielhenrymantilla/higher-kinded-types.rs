crate::utils::cfg_match! {
    feature = "fn_traits" => (
        dispatch! {$
            fn_traits = true
        }
    );
    _ => (
        dispatch! {$
            fn_traits = false
        }
    );
}

macro_rules! dispatch {($_:tt
    fn_traits =
        $(true $($if_cfg_fn_traits:tt)?)?
        $(false $($if_not_cfg_fn_traits:tt)?)?
) => (
    /// Produce <code>impl [ForLt]</code>[^auto] types _on demand_.
    ///
    /// [ForLt]: trait@crate::ForLt
    /// [^auto]: `+ Send + Sync + Unpin`
    ///
    /// ### Syntax
    ///
    ///   - #### Full syntax
    ///
    ///     ```rust
    ///     # use ::higher_kinded_types::ForLt;
    ///     # mod some { pub use ::std::borrow::Cow as Arbitrary; }
    ///     # use str as Type; let _:
    ///     ForLt!(<'r> = some::Arbitrary<'r, Type>)
    ///     # ;
    ///     ```
    ///
    ///   - #### Shorthand syntax
    ///
    ///     You can use the anonymous/elided `'_` lifetime (or even implicitly
    ///     elided if behind `&`) in which case you skip the `<'lt> =` part, and
    ///     just write:
    ///
    ///     ```rust
    ///     # use ::higher_kinded_types::ForLt;
    ///     # mod some { pub use ::std::borrow::Cow as Arbitrary; }
    ///     # use str as Type; let _:
    ///     ForLt!(some::Arbitrary<'_, Type>)
    ///     # ;
    ///     ```
    ///
    /// ### Examples
    ///
    /// ```rust
    /// use ::higher_kinded_types::prelude::{*, ForLt};
    ///
    /// type A = ForLt!(<'r> = &'r str);
    /// // the following two definitions are equivalent to A (syntax sugar).
    /// type B = ForLt!(&'_ str);
    /// type C = ForLt!(&str);
    ///
    /// //     Let `'r` be `'static`, this results in:
    /// //                      |
    /// //                      vvvvvvv
    /// let a: <A as ForLt>::Of<'static> = "a";
    /// //     ^^^^^^^^^^^^^^^^^^^^^^^^^
    /// //          `&'static str` !
    /// //     vvvvvvvvvvvvvvvvvvvvvvvvv
    /// let b: <B as ForLt>::Of<'static> = "b";
    /// let c: <C as ForLt>::Of<'static> = "c";
    /// ```
    #[macro_export] #[doc(hidden)]
    macro_rules! ඞForLt {
        (
            // Named lifetime case: e.g. `ForLt!(<'r> = &'r str)`.
            <$lt:lifetime> = $T:ty $_(,)?
        ) => (
            $($($if_cfg_fn_traits)?
                $_ crate::ඞ::ForLt<
                    for<$lt> fn($_ crate::ඞ::__<$lt>) -> $T
                >
            )?
            $($($if_not_cfg_fn_traits)?
                $_ crate::ඞ::ForLt<
                    dyn for<$lt> $_ crate::advanced::WithLifetime<$lt, Of = $T>,
                >
            )?
        );

        (
            // default case: as if we had `ForLt!(<'_> = $($input)*)`.
            // For instance: `ForLt!(&str)` or `ForLt!(&'_ str)`.
            $_($shorthand_syntax:tt)*
        ) => (
            $($($if_cfg_fn_traits)?
                $_ crate::ඞ::ForLt<
                    fn($_ crate::ඞ::r#for<'_>) -> $_($shorthand_syntax)*
                >
            )?
            $($($if_not_cfg_fn_traits)?
                $_ crate::ForLt! {
                    <'ඞ /* ' */> = $_ crate::ඞForLt_munch! {
                        [output: ]
                        [input: $_($shorthand_syntax)*]
                        [mode: default]
                    }
                }
            )?
        );
    }
    /// ```rust
    /// type A = ::higher_kinded_types::ForLt![()];
    /// type B = ::higher_kinded_types::prelude::ForLt![()];
    /// ```
    #[doc(inline)]
    pub use ඞForLt as ForLt;
)}
use dispatch;

/// <-- rust-analyzer nudge
/// ```rust ,ignore
/// ඞForLtWF![];
/// ``` -->
#[macro_export] #[doc(hidden)] /** Not part of the public API */
macro_rules! ඞForLtWF {
    (
        <$lt:lifetime> = $T:ty $(,)?
    ) => (
        $crate::ඞ::ForLt<
            dyn for<$lt> $crate::advanced::WithLifetime<$lt, Of = $T>,
            <() as $crate::ඞ::IdentityIgnoring<
                $crate::ඞ::map_lifetime![$lt => 'static in $T]
            >>::ItSelf
        >
    );

    (
        $T:ty $(,)?
    ) => (
        $crate::ඞ::ForLt<
            dyn for<'ඞ/*'*/> $crate::advanced::WithLifetime<'ඞ/*'*/,
                Of = $crate::ඞ::map_lifetime!['_ => 'ඞ/*'*/ in $T],
            >,
            <() as $crate::ඞ::IdentityIgnoring<
                $crate::ඞ::map_lifetime!['_ => 'static in $T]
            >>::ItSelf,
        >
    );
}

#[doc(inline)]
pub use ඞForLtWF as ForLtWF;

#[test]
fn soundness_test() {
    let local = String::from("…");
    let _: &'static str = demo(&local);

    fn demo<'r>(r: &'r str) -> &'static str {
     // drop(None::<ForLtWF![&&'r i32]>);
     // drop(None::<ForLtWF![& &'r i32]>);
     // drop(None::<ForLtWF![&'_ &'r i32]>);
     // drop(None::<ForLtWF![<'any> = &'any &'r ()]>);

        let local = String::from("…");
        let _: &'static str = demo2::<ForLtWF![<'any> = &'any &'_ ()]>(&local);

        let _: &'static str = demo2::<ForLt![<'any> = &'any &'r ()]>(r);
     // let _: &'static str = demo2::<ForLtWF![<'any> = &'any &'r ()]>(b);

        let s: &'static str = demo3::<ForLtWF![<'any> = &'static &'any ()]>(r);
        s
    }

    trait Outlives<'a, 'b> : Sized {
        fn outlives<T : ?Sized>(_: [Self; 0], r: &'a T) -> &'b T;
    }
    impl<'a, 'b> Outlives<'a, 'b> for &'b &'a () {
        fn outlives<T : ?Sized>(_: [Self; 0], r: &'a T) -> &'b T { r }
    }

    fn demo2<'r, T : crate::ForLt>(
        r: &'r str,
    ) -> &'static str
    where
    
        for<'any> <T as crate::ForLt>::Of<'any> : Outlives<'r, 'any>,
    {
        <T::Of<'static> as Outlives<'r, 'static>>::outlives([], r)
    }

    fn demo3<'r, 'b, T : crate::ForLt>(
        r: &'r str,
    ) -> &'static str
    where
        // for 1.76.0 test; TODO: put inline bounds back.
        for<'any> <T as crate::ForLt>::Of<'any> : Outlives<'any, 'static>,
    {
        <T::Of<'r> as Outlives<'r, 'static>>::outlives([], r)
    }
}
