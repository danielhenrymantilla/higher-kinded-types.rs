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
