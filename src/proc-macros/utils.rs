use super::*;

macro_rules! bail {( $msg:expr => $spannee:expr $(,)? ) => (
    return Err(Error::new_spanned(&$spannee, $msg))
)}
pub(crate) use bail;

use ::core::ops::Range;

pub(crate)
trait SpanRange {
    fn span_range(self) -> Range<Span>;
}

impl SpanRange for Span {
    fn span_range(self) -> Range<Span> { self..self }
}

impl SpanRange for Range<Span> {
    fn span_range(self) -> Range<Span> { self }
}

impl SpanRange for TokenStream2 {
    fn span_range(self) -> Range<Span> {
        let mut spans = self.into_iter().map(|tt| tt.span());
        let first = spans.next().unwrap_or_else(|| Span::mixed_site());
        let last = spans.fold(first, |_, current| current);
        first..last
    }
}

pub(crate)
fn compile_warning(spans: impl SpanRange, message: &str) -> TokenStream2 {
    let Range { start, end } = spans.span_range();
    let warn_deprecated = quote_spanned!(Span::mixed_site()=>
        #[warn(deprecated)]
    );
    let start = quote_spanned!(start=>
        #[deprecated = #message]
        #[allow(nonstandard_style)]
        struct proc_macro_warning {}

        #warn_deprecated
        _ = proc_macro_warning
    );
    let end = quote_spanned!(end=>
        #[allow(warnings, clippy::all)]
        const _: () = {
            #start {};
        };
    );
    end
}

pub(crate)
fn maybe_debug(ts: TokenStream2) -> TokenStream2 {
    if cfg!(feature = "debug-macros") {
        eprintln!("\n[debug-macros expansion]\n\n{}", ts.to_token_stream());
    }
    ts
}
