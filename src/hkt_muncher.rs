#[doc(hidden)] /** Not part of the public API */ #[macro_export]
macro_rules! ඞForLt_munch {
    // case `'_`
    (
        [output:
            $($acc:tt)*
        ]
        [input:
            '_
            $($rest:tt)*
        ]
        $mode:tt
    ) => ($crate::ඞForLt_munch! {
        [output:
            $($acc)*
            'ඞ /* ' */
        ]
        [input:
            $($rest)*
        ]
        $mode
    });

    // case `&'lifetime` (including `&'_`)
    (
        [output:
            $($acc:tt)*
        ]
        [input:
            &
            $lifetime:lifetime
            $($rest:tt)*
        ]
        $mode:tt
    ) => ($crate::ඞForLt_munch! {
        [output:
            $($acc)*
            &
        ]
        [input:
            $lifetime
            $($rest)*
        ]
        $mode
    });

    // case `& /* no lifetime */` (implicit elision)
    (
        $acc:tt
        [input:
            &
            $($rest:tt)*
        ]
        $mode:tt
    ) => ($crate::ඞForLt_munch! {
        $acc
        [input:
            // make it explicit
            &'_
            $($rest)*
        ]
        $mode
    });

    // case `(…)` (need to deep recurse)
    (
        [output:
            $($acc:tt)*
        ]
        [input:
            ( $($group:tt)* )
            $($rest:tt)*
        ]
        $mode:tt
    ) => ($crate::ඞForLt_munch! {
        [output:
            $($acc)*
            $crate::ඞForLt_munch! {
                [output: ]
                [input: $($group)*]
                [mode: parenthesized]
            }
        ]
        [input:
            $($rest)*
        ]
        $mode
    });

    // case `[…]` (need to deep recurse)
    (
        [output:
            $($acc:tt)*
        ]
        [input:
            [ $($group:tt)* ]
            $($rest:tt)*
        ]
        $mode:tt
    ) => ($crate::ඞForLt_munch! {
        [output:
            $($acc)*
            $crate::ඞForLt_munch! {
                [output: ]
                [input: $($group)*]
                [mode: square_bracketed]
            }
        ]
        [input:
            $($rest)*
        ]
        $mode
    });

    /* No need to recurse into `{ … }`, so we handle it with the default tt */

    // Otherwise / default `tt` case: just forward it, _verbatim_
    (
        [output:
            $($acc:tt)*
        ]
        [input:
            $otherwise:tt
            $($rest:tt)*
        ]
        $mode:tt
    ) => ($crate::ඞForLt_munch! {
        [output:
            $($acc)*
            $otherwise
        ]
        [input:
            $($rest)*
        ]
        $mode
    });

    /* END OF RECURSION */
    (
        [output: $Output:ty $(,)? ]
        [input: /* nothing left! */]
        [mode: default]
    ) => (
        $Output
    );
    // Also handle the grouping modes:
    (
        [output: $($output:tt)*]
        [input: /* nothing left! */]
        [mode: parenthesized]
    ) => (
        ( $($output)* )
    );
    (
        [output: $($output:tt)*]
        [input: /* nothing left! */]
        [mode: square_bracketed]
    ) => (
        [ $($output)* ]
    );
}
