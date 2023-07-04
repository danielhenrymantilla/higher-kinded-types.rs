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
    /// use ::higher_kinded_types::ForLt;
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
    #[macro_export]
    macro_rules! ForLt {
        (
            // Named lifetime case: e.g. `ForLt!(<'r> = &'r str)`.
            <$lt:lifetime> = $T:ty $_(,)?
        ) => (
            $($($if_cfg_fn_traits)?
                $_ crate::ඞ::ForLt<
                    for<$lt> fn($_ crate::ඞ::Of<$lt>) -> $T
                >
            )?
            $($($if_not_cfg_fn_traits)?
                $_ crate::ඞ::ForLt<
                    dyn for<$lt> $_ crate::ඞ::WithLifetime<$lt, T = $T>,
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
)}
use dispatch;
