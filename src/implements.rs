use std::collections::HashSet;

use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse::{ParseStream, Parser as _},
    spanned::Spanned,
    Arm, Error, Expr, ExprLit, ExprMatch, Lit, Pat, PatIdent, PatLit, PatOr, Result,
};

enum IdentOrWild {
    Ident(Ident),
    Wild,
}

enum ParseStrPattern {
    Triple(Vec<u8>, IdentOrWild, Vec<u8>),
    Single(Vec<u8>),
}

fn parse_str_pattern(s: &str, span: Span) -> Result<ParseStrPattern> {
    let mut a = String::new();
    let mut i = String::new();
    let mut b = String::new();
    let mut chars = s.chars();
    let mut prev = None;
    #[allow(clippy::while_let_on_iterator)]
    while let Some(ch) = chars.next() {
        match (prev, ch) {
            (Some('}'), '}') => {
                a.push('}');
                prev = None;
            }
            (Some('{'), '}') => {
                return Err(Error::new(
                    span,
                    r"str-match: invalid format string: expected identity but `}` found
if you intended to match `{}`, you can escape it using `{{}}`",
                ));
            }
            (Some('{'), '{') => {
                a.push(ch);
                prev = None;
            }
            (Some('}'), _) => {
                return Err(Error::new(
                    span,
                    r"str-match: invalid format string: unmatched `}` found
if you intended to match `}`, you can escape it using `}}`",
                ));
            }
            (Some('{'), _) => {
                i.push(ch);
                break;
            }
            (_, '}') => {
                prev = Some('}');
            }
            (_, '{') => {
                prev = Some('{');
            }
            _ => {
                a.push(ch);
            }
        }
    }
    // format!("}");
    // format!("{}");
    // format!("{a}");
    // format!("{a");
    // if let [a @ .., b @ ..] = &[0] {}
    Ok(if prev == Some('{') {
        let mut is_terminated = true;
        #[allow(clippy::while_let_on_iterator)]
        while let Some(ch) = chars.next() {
            if ch == '}' {
                is_terminated = false;
                break;
            }
            i.push(ch);
        }
        if is_terminated {
            return Err(Error::new(
                span,
                r"str-match: invalid format string: expected `'}'` but string was terminated
if you intended to match `{`, you can escape it using `{{`",
            ));
        }
        prev = None;
        for ch in chars {
            match (prev, ch) {
                (Some('}'), '}') => {
                    b.push('}');
                    prev = None;
                }
                (Some('}'), _) => {
                    return Err(Error::new(
                        span,
                        r"str-match: invalid format string: unmatched `}` found
if you intended to match `}`, you can escape it using `}}`",
                    ));
                }
                (_, '}') => {
                    prev = Some('}');
                }
                (Some('{'), '{') => {
                    b.push(ch);
                    prev = None;
                }
                (Some('{'), _) => {
                    return Err(Error::new(
                        span,
                        "`{}` can only be used once per str pattern",
                    ))
                }
                (_, '{') => {
                    prev = Some('{');
                }
                _ => {
                    b.push(ch);
                }
            }
        }
        let i = if i == "_" {
            IdentOrWild::Wild
        } else {
            IdentOrWild::Ident(Ident::new(&i, span))
        };
        ParseStrPattern::Triple(a.into(), i, b.into())
    } else {
        ParseStrPattern::Single(a.into())
    })
}

fn convert_pat(pat: &Pat, set: &mut HashSet<Ident>) -> Result<TokenStream> {
    match pat {
        Pat::Lit(PatLit { attrs, expr }) => {
            let expr = if let Expr::Lit(ExprLit {
                attrs,
                lit: Lit::Str(s),
            }) = expr.as_ref()
            {
                let v = s.value();
                match parse_str_pattern(&v, s.span())? {
                    ParseStrPattern::Triple(a, IdentOrWild::Ident(i), b) => {
                        set.insert(i.clone());
                        quote! {
                            #(#attrs)* [#(#a,)* #i @ .., #(#b,)*]
                        }
                    }
                    ParseStrPattern::Triple(a, IdentOrWild::Wild, b) => {
                        quote! {
                            #(#attrs)* [#(#a,)* .., #(#b,)*]
                        }
                    }
                    ParseStrPattern::Single(s) => quote! {
                        #(#attrs)* [#(#s),*]
                    },
                }
            } else {
                expr.to_token_stream()
            };
            Ok(quote! {
                #(#attrs)* #expr
            })
        }
        Pat::Ident(PatIdent {
            attrs,
            by_ref: None,
            mutability: None,
            ident,
            subpat: None,
        }) => {
            set.insert(ident.clone());
            Ok(quote! {
                #(#attrs)* #ident
            })
        }
        Pat::Ident(_) => Err(Error::new(
            pat.span(),
            "str-match: complex pattern is currently unsupported",
        )),
        Pat::Or(PatOr {
            attrs,
            leading_vert,
            cases,
        }) => {
            let mut c = Vec::with_capacity(cases.len());
            for case in cases {
                c.push(convert_pat(case, set)?);
            }
            Ok(quote! {
                #(#attrs)* #leading_vert #(#c)|*
            })
        }
        p => Ok(p.to_token_stream()),
    }
}

fn str_match_impl(input: ParseStream) -> Result<TokenStream> {
    let ExprMatch {
        attrs,
        match_token,
        expr,
        arms,
        ..
    } = input.parse::<ExprMatch>()?;
    let mut a = Vec::with_capacity(arms.len());
    for Arm {
        attrs,
        pat,
        guard,
        fat_arrow_token,
        body,
        ..
    } in arms
    {
        let mut set = HashSet::new();
        let pat = convert_pat(&pat, &mut set)?;
        let mut idents = set.iter().collect::<Vec<_>>();
        idents.sort();
        let guard = guard.as_ref().map(|(if_, expr)| quote! {
            #if_ {
                #(#[allow(unused)] let #idents = unsafe { ::core::str::from_utf8_unchecked(#idents) };)*
                #expr
            }
        });
        a.push(quote! {
            #(#attrs)*
            #pat #guard #fat_arrow_token {
                #(#[allow(unused)] let #idents = unsafe { ::core::str::from_utf8_unchecked(#idents) };)*
                #body
            }
        });
    }
    Ok(quote! {
        #(#[#attrs])*
        #match_token str::as_bytes(#expr) {
            #(#a)*
        }
    })
}

#[cfg(feature = "attribute")]
pub(crate) fn str_match_attr(_attrs: TokenStream, tokens: TokenStream) -> TokenStream {
    str_match_impl
        .parse2(tokens)
        .unwrap_or_else(|e| Error::to_compile_error(&e))
}

#[cfg(any(not(feature = "attribute"), test))]
pub(crate) fn str_match_macro(tokens: TokenStream) -> TokenStream {
    str_match_impl
        .parse2(tokens)
        .unwrap_or_else(|e| Error::to_compile_error(&e))
}
#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test() {
        assert_eq!(
            str_match_macro(quote! {
                match a {
                    "a" => {}
                    "a{b}c" => {}
                    "{{" => {}
                    "}}" => {}
                    "{_}" => {}
                    "{d}" if d.starts_with("e") => {}
                    e => {}
                    _ => {}
                }
            })
            .to_string(),
            quote! {
                match str::as_bytes(a) {
                    [97u8] => {{}}
                    [97u8, b @ .., 99u8,] => {
                        #[allow(unused)]
                        let b = unsafe{::core::str::from_utf8_unchecked(b)};
                        {}
                    }
                    [123u8] => {{}}
                    [125u8] => {{}}
                    [..,] => {{}}
                    [d @ ..,] if {
                        #[allow(unused)]
                        let d = unsafe{::core::str::from_utf8_unchecked(d)};
                        d.starts_with("e")
                    }=> {
                        #[allow(unused)]
                        let d = unsafe{::core::str::from_utf8_unchecked(d)};
                        {}
                    }
                    e => {
                        #[allow(unused)]
                        let e = unsafe{::core::str::from_utf8_unchecked(e)};
                        {}
                    }
                    _ => {{}}
                }
            }
            .to_string()
        )
    }
}
