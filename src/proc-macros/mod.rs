#![allow(unused_imports)]
use ::core::ops::Not as _;
use ::proc_macro::TokenStream;
use ::proc_macro2::{*, TokenStream as TokenStream2, TokenTree as TT};
use ::quote::{
    format_ident,
    quote, quote_spanned,
    ToTokens,
};
use ::syn::{*,
    parse::{Parse, Parser, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    Result, // explicitly shadow it
};

use crate::utils::compile_warning;

mod utils;

#[proc_macro] pub
fn map_lifetime(input: TokenStream) -> TokenStream {
    map_lifetime_inner(input.into())
        .map_err(|mut error| {
            let mut errors =
                error
                    .into_iter()
                    .map(|error| Error::new_spanned(
                        error.to_compile_error(),
                        format!("`map_lifetime!`: {error}"),
                    ))
            ;
            error = errors.next().unwrap();
            error.extend(errors);
            error
        })
        .map(utils::maybe_debug)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

fn map_lifetime_inner(input: TokenStream2) -> Result<TokenStream2> {
    let Input { ref src, _arrow, dst, _in, tokens } = parse2(input)?;
    let src_ident = &src.ident.to_string();
    let ret = replace_lifetime((src, src_ident), dst, tokens)?;
    Ok(ret)
}

struct Input {
    src: Lifetime,
    _arrow: Token![=>],
    dst: Lifetime,
    _in: Token![in],
    tokens: TokenStream2,
}

impl Parse for Input {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        Ok(Self {
            src: input.parse()?,
            _arrow: input.parse()?,
            dst: input.parse()?,
            _in: input.parse()?,
            tokens: input.parse().unwrap(),
        })
    }
}

fn replace_lifetime(
    src: (&Lifetime, &str),
    mut dst: Lifetime,
    tokens: TokenStream2,
) -> Result<TokenStream2> {
    let mut tts = (Box::new(tokens.into_iter()) as Box<dyn Iterator<Item = _>>).peekable();
    let mut ret = TokenStream2::new();
    while let Some(mut tt) = tts.next() {
        tt = match tt {
            // subrecurse
            TT::Group(mut g) => {
                let span = g.span();
                g = Group::new(g.delimiter(), replace_lifetime(src, dst.clone(), g.stream())?);
                g.set_span(span);
                g.into()
            },
            // `macro!`
            TT::Ident(ref _macro) if matches!(
                tts.peek(), Some(TT::Punct(p))
                if p.as_char() == '!' && p.spacing() == Spacing::Alone
            ) => {
                utils::bail!("macro invocations are not supported" => tt)
            },
            // `'lifetime | '_`
            TT::Punct(p)
                if p.as_char() == '\'' && p.spacing() == Spacing::Joint
                && (
                    matches!(tts.peek(), Some(TT::Ident(_)))
                    ||
                    matches!(
                        tts.peek(),
                        Some(TT::Punct(p)) if p.as_char() == '_' && p.spacing() == Spacing::Alone
                    )
                )
            => {
                let lt_name = &format!("'{}", tts.next().unwrap());
                let lifetime = &Lifetime::new(lt_name, p.span());
                if &lt_name[1..] == src.1 {
                    dst.ident.set_span(dst.ident.span().located_at(p.span()));
                    &dst
                } else {
                    lifetime
                }.to_tokens(&mut ret);
                continue;
            }
            // `&` not followed by a lifetime
            // Note: this can have false positives with stuff such as `Type<{ let x = &42; 0 }>`.
            // So be it, perhaps I'll use `syn` parsing at some point.
            TT::Punct(ref amp)
                if src.1 == "_"
                && amp.as_char() == '&' // && amp.spacing() == Spacing::Alone
                && matches!(tts.peek(), Some(TT::Punct(ap)) if ap.as_char() == '\'').not()
            => {
                // Let's not forget the `&` in question
                amp.to_tokens(&mut ret);
                dst.ident.set_span(dst.ident.span().located_at(amp.span()));
                dst.to_tokens(&mut ret);
                continue;
            }
            _ => tt,
        };
        tt.to_tokens(&mut ret);
    }
    Ok(ret.into_iter().collect())
}