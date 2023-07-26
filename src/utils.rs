macro_rules! cfg_match {
    (
        _ => ( $($expansion:tt)* ) ;
    ) => (
        $($expansion)*
    );

    (
        $cfg:meta => $expansion:tt;
        $($($else:tt)+)?
    ) => (
        #[cfg($cfg)]
        $crate::utils::cfg_match! {
            _ => $expansion;
        } $(

        #[cfg(not($cfg))]
        $crate::utils::cfg_match! {
            $($else)+
        } )?
    );

    ({ $($input:tt)* }) => ({
        $crate::utils::cfg_match! { $($input)* }
    });
}
pub(in crate) use cfg_match;

macro_rules! macro_export_ {(
    $( #$attr:tt )*
    macro_rules! $macro_name:ident $macro_rules:tt
) => (
    ::paste::paste! {
        #[doc(hidden)] #[macro_export]
        macro_rules! [< ඞ $macro_name >] $macro_rules

        #[doc(inline)]
        $( #$attr )*
        pub use [< ඞ $macro_name >] as $macro_name;
    }
)}
pub(in crate) use macro_export_ as macro_export;
