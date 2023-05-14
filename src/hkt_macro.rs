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
    /// Produce <code>impl [HKT]</code> types _on demand_.
    ///
    /// [HKT]: trait@crate::HKT
    ///
    /// ### Syntax
    ///
    ///   - #### Full syntax
    ///
    ///     ```rust
    ///     # use ::higher_kinded_types::HKT;
    ///     # mod some { pub use ::std::borrow::Cow as Arbitrary; }
    ///     # use str as Type; let _:
    ///     HKT!(<'r> = some::Arbitrary<'r, Type>);
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
    ///     # use ::higher_kinded_types::HKT;
    ///     # mod some { pub use ::std::borrow::Cow as Arbitrary; }
    ///     # use str as Type; let _:
    ///     HKT!(some::Arbitrary<'_, Type>);
    ///     # ;
    ///     ```
    ///
    /// ### Examples
    ///
    /// ```rust
    /// use ::higher_kinded_types::*;
    ///
    /// type A = HKT!(<'r> = &'r str);
    /// // the following two definitions are equivalent to A (syntax sugar).
    /// type B = HKT!(&'_ str);
    /// type C = HKT!(&str);
    ///
    /// //            let `'r` be `'static`; this results in…
    /// //                    vvvvvvv
    /// let a: <A as HKT>::__<'static> = "a";
    /// //     ^^^^^^^^^^^^^^^^^^^^^^^
    /// //        … `&'static str` !
    /// //     vvvvvvvvvvvvvvvvvvvvvvv
    /// let b: <B as HKT>::__<'static> = "b";
    /// let c: <C as HKT>::__<'static> = "c";
    /// ```
    #[macro_export]
    macro_rules! HKT {
        (
            // Named lifetime case: e.g. `HKT!(<'r> = &'r str)`.
            <$lt:lifetime> = $T:ty $_(,)?
        ) => (
            $($($if_cfg_fn_traits)?
                $_ crate::ඞ::HKT<
                    for<$lt> fn($_ crate::ඞ::__<$lt>) -> $T
                >
            )?
            $($($if_not_cfg_fn_traits)?
                $_ crate::ඞ::HKT<
                    dyn for<$lt> $_ crate::ඞ::WithLifetime<$lt, T = $T>,
                >
            )?
        );

        (
            // default case: as if we had `HKT!(<'_> = $($input)*)`.
            // For instance: `HKT!(&str)` or `HKT!(&'_ str)`.
            $_($input:tt)*
        ) => (
            $($($if_cfg_fn_traits)?
                $_ crate::ඞ::HKT<
                    fn($_ crate::ඞ::r#for<'_>) -> $_($input)*
                >
            )?
            $($($if_not_cfg_fn_traits)?
                $_ crate::HKT! {
                    <'ඞ /* ' */> = $_ crate::ඞHKT_munch! {
                        [output: ]
                        [input: $_($input)*]
                        [mode: default]
                    }
                }
            )?
        );
    }
)}
use dispatch;

pub use HKT;
