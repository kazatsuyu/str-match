use proc_macro::TokenStream;

mod implements;

#[cfg(feature = "attribute")]
#[proc_macro_attribute]
pub fn str_match(attrs: TokenStream, tokens: TokenStream) -> TokenStream {
    implements::str_match_attr(attrs.into(), tokens.into()).into()
}

#[cfg(not(feature = "attribute"))]
#[proc_macro]
pub fn str_match(tokens: TokenStream) -> TokenStream {
    implements::str_match_macro(tokens.into()).into()
}
