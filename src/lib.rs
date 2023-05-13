#![doc = include_str!("../README.md")]
#![no_std]
#![forbid(unsafe_code)]

use {
    ::core::{
        ops::Not as _,
    },
};

#[cfg(COMMENTED_OUT)] // <- Remove this when used!
/// The crate's prelude.
pub
mod prelude {
    // …
}

// macro internals
#[doc(hidden)] /** Not part of the public API */ pub
mod ඞ {
    pub use ::core; // or `std`
}

#[doc = include_str!("compile_fail_tests.md")]
mod _compile_fail_tests {}
