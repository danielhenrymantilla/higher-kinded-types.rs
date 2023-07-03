//! Niche and advanced, you can ignore this to begin with.
//! Module for other [`ForLt`]-like constructs: extended to various "arities".
//!
//! For instance:
//!   - [`ForTy`] so as to be generic over a type parameter,
//!   - [`ForLtAndLt`] so as to be generic over _two_ lifetime parameters,
//!   - [`ForLtAndTy`] so as to be generic over a lifetime parameter and a type
//!     parameter.

#[doc(no_inline)]
pub use crate::ForLt;

pub trait ForTy {
    type Of<T>;
}

pub trait ForLtAndLt {
    type Of<'a, 'b>;
}

pub trait ForUnsizedTy {
    type Of<T : ?Sized>;
}

pub trait ForLtAndTy {
    type Of<'lt, T>;
}

#[doc(hidden)]
#[macro_export]
macro_rules! __For {
    (
        <$lt:lifetime> = $Type:ty $(,)?
        // Produces an `impl ForLt`
    ) => (
        $crate::For! { <$lt> = $Type }
    );

    (
        <$a:lifetime, $b:lifetime> = $Type:ty $(,)?
        // Produces an `impl ForLtAndLt`
    ) => (
        ::core::compile_error! { "not implemented yet" }
    );

    (
        <$T:ident> = $Type:ty $(,)?
        // Produces an `impl ForTy`
    ) => (
        ::core::compile_error! { "not implemented yet" }
    );

    (
        <$lt:lifetime, $T:ident> = $Type:ty $(,)?
        // Produces an `impl ForLtAndTy`
    ) => (
        ::core::compile_error! { "not implemented yet" }
    );
}

#[doc(inline)]
pub use __For as For;
