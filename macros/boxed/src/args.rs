use syn::{
    Error, Result, Token,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

use crate::utils::parse_list;

pub mod kw {
    syn::custom_keyword!(bounds);
    syn::custom_keyword!(pin);
}

pub struct Args {
    pub pin: Option<kw::pin>,
    pub dyn_: Option<Token![dyn]>,
    pub bounds: Punctuated<syn::TypeParamBound, Token![+]>,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut pin = None;
        let mut dyn_ = None;
        let mut bounds = Punctuated::new();

        parse_list(
            input,
            |input| {
                let lookahead = input.lookahead1();

                if lookahead.peek(kw::pin) {
                    let t = input.parse::<kw::pin>()?;

                    if pin.is_some() {
                        return Err(Error::new_spanned(t, "duplicate `pin` argument"));
                    }

                    pin = Some(t);
                } else if lookahead.peek(Token![dyn]) {
                    let t = input.parse::<Token![dyn]>()?;

                    if dyn_.is_some() {
                        return Err(Error::new_spanned(t, "duplicate `dyn` argument"));
                    }

                    dyn_ = Some(t);
                } else if lookahead.peek(kw::bounds) {
                    let t = input.parse::<kw::bounds>()?;

                    if !bounds.is_empty() {
                        return Err(Error::new_spanned(t, "duplicate `bounds` argument"));
                    }

                    bounds = Punctuated::parse_separated_nonempty(input)?;
                } else {
                    return Err(lookahead.error());
                }

                Ok(())
            },
            Token![,],
        )?;

        Ok(Self { pin, dyn_, bounds })
    }
}
