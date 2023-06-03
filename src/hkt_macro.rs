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
    /// Produce <code>impl [Gat]</code> types _on demand_.
    ///
    /// [Gat]: trait@crate::Gat
    ///
    /// ### Syntax
    ///
    ///   - #### Full syntax
    ///
    ///     ```rust
    ///     # use ::higher_kinded_types::Gat;
    ///     # mod some { pub use ::std::borrow::Cow as Arbitrary; }
    ///     # use str as Type; let _:
    ///     Gat!(<'r> = some::Arbitrary<'r, Type>);
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
    ///     # use ::higher_kinded_types::Gat;
    ///     # mod some { pub use ::std::borrow::Cow as Arbitrary; }
    ///     # use str as Type; let _:
    ///     Gat!(some::Arbitrary<'_, Type>);
    ///     # ;
    ///     ```
    ///
    /// ### Examples
    ///
    /// ```rust
    /// use ::higher_kinded_types::*;
    ///
    /// type A = Gat!(<'r> = &'r str);
    /// // the following two definitions are equivalent to A (syntax sugar).
    /// type B = Gat!(&'_ str);
    /// type C = Gat!(&str);
    ///
    /// //            let `'r` be `'static`; this results in…
    /// //                    vvvvvvv
    /// let a: <A as Gat>::__<'static> = "a";
    /// //     ^^^^^^^^^^^^^^^^^^^^^^^
    /// //        … `&'static str` !
    /// //     vvvvvvvvvvvvvvvvvvvvvvv
    /// let b: <B as Gat>::__<'static> = "b";
    /// let c: <C as Gat>::__<'static> = "c";
    /// ```
    #[macro_export]
    macro_rules! Gat {
        (
            // Named lifetime case: e.g. `Gat!(<'r> = &'r str)`.
            <$lt:lifetime> = $T:ty $_(,)?
        ) => (
            $($($if_cfg_fn_traits)?
                $_ crate::ඞ::Gat<
                    for<$lt> fn($_ crate::ඞ::__<$lt>) -> $T
                >
            )?
            $($($if_not_cfg_fn_traits)?
                $_ crate::ඞ::Gat<
                    dyn for<$lt> $_ crate::ඞ::WithLifetime<$lt, T = $T>,
                >
            )?
        );

        (
            // default case: as if we had `Gat!(<'_> = $($input)*)`.
            // For instance: `Gat!(&str)` or `Gat!(&'_ str)`.
            $_($input:tt)*
        ) => (
            $($($if_cfg_fn_traits)?
                $_ crate::ඞ::Gat<
                    fn($_ crate::ඞ::r#for<'_>) -> $_($input)*
                >
            )?
            $($($if_not_cfg_fn_traits)?
                $_ crate::Gat! {
                    <'ඞ /* ' */> = $_ crate::ඞGat_munch! {
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

pub use Gat;
