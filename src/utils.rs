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

cfg_match! {
    feature = "better-docs" => (
        macro_rules! macro_export__ {( $($input:tt)* ) => (
            #[::macro_pub::macro_pub]
            $($input)*
        )}
    );

    _ => (
        macro_rules! macro_export__ {(
         $( #$attr:tt )*
            macro_rules! $macro_name:ident $body:tt
        ) => (
                #[macro_export]
             $( #$attr )*
                macro_rules! $macro_name $body

                pub(in crate) use $macro_name;

        )}
    );
}
pub(in crate) use macro_export__ as macro_export;
