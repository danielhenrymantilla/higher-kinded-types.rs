crate::utils::cfg_match! {
    feature = "fn_traits" => (
        dispatch! {$
            fn_traits = yes
        }
    );
    _ => (
        dispatch! {$
            fn_traits = no
        }
    );
}

macro_rules! dispatch {($_:tt
    fn_traits =
        $(yes $($if_cfg_fn_traits:tt)?)?
        $(no $($if_not_cfg_fn_traits:tt)?)?
) => (
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
                $_ crate::ඞ::PhantomData<
                    for<$lt> fn($_ crate::ඞ::__<$lt>) -> $T
                >
            )?
            $($($if_not_cfg_fn_traits)?
                $_ crate::ඞ::PhantomData<
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
                $_ crate::ඞ::PhantomData<
                    fn($_ crate::ඞ::For<'_>) -> $_($input)*
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
