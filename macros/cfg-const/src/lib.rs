//! Conditionally make a function `const`.
//!
//! See [`cfg_const`].

use proc_macro2::{Span, TokenStream, TokenTree};
use quote::{quote, quote_spanned};
use syn::{
    Error, Result, Token,
    parse::{Parse, ParseStream},
    spanned::Spanned,
    token::{Bracket, Paren},
};

use macro_helpers::{error::gather_errors_and_return, fail};

/// Conditionally make a function `const`.
///
/// ```ignore
/// #[cfg_const(feature = "nightly")]
/// pub fn foo() -> i32 { .. }
/// // will expand to:
/// //   #[cfg(feature = "nightly")]
/// //   pub const fn foo() -> i32 { .. }
/// //   #[cfg(not(feature = "nightly"))]
/// //   pub fn foo() -> i32 { .. }
/// ```
#[proc_macro_attribute]
pub fn cfg_const(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let args = syn::parse_macro_input!(args);
    let input = syn::parse_macro_input!(input);

    let (output, error) = gather_errors_and_return(|| cfg_const_impl(args, input));
    let error = error.map(Error::into_compile_error);
    quote! { #output #error }.into()
}

fn cfg_const_impl(cfg: Cfg, input: syn::Item) -> TokenStream {
    match input {
        syn::Item::Fn(input) => {
            if input.sig.constness.is_some() {
                fail!(
                    input.sig.constness.span(),
                    "function is already `const`",
                    return input,
                );
            }

            let mut const_input = input.clone();
            const_input.attrs.push(cfg.to_cfg());
            const_input.sig.constness = Some(Token![const](cfg.span()));

            let mut non_const_input = input;
            non_const_input.attrs.push(cfg.not().to_cfg());

            quote! { #const_input #non_const_input }
        }

        _ => fail!(
            Span::call_site(),
            "`#[cfg_const]` can only be applied to functions",
            return input,
        ),
    }
}

struct Cfg {
    args: TokenStream,
}

impl Cfg {
    fn span(&self) -> Span {
        Span::call_site()
    }

    fn not(&self) -> Self {
        let args = &self.args;
        Self {
            args: quote_spanned! {self.span()=>
                not(#args)
            },
        }
    }

    fn to_cfg(&self) -> syn::Attribute {
        let span = self.span();
        syn::Attribute {
            pound_token: Token![#](span),
            style: syn::AttrStyle::Outer,
            bracket_token: Bracket(span),
            meta: syn::Meta::List(syn::MetaList {
                path: syn::Ident::new("cfg", span).into(),
                delimiter: syn::MacroDelimiter::Paren(Paren(span)),
                tokens: self.args.clone(),
            }),
        }
    }
}

impl Parse for Cfg {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut args = TokenStream::new();

        while !input.is_empty() {
            if input.peek(Token![,]) {
                return Err(input.error("unexpected comma"));
            }

            args.extend([input.parse::<TokenTree>()?]);
        }

        Ok(Self { args })
    }
}
