use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, TokenStreamExt, format_ident, quote, quote_spanned};
use syn::{
    Error, GenericParam, Generics, Ident, Result, Token, Type, TypeImplTrait, TypeNever, TypeParam,
    braced,
    parse::{Parse, ParseStream},
    parse_quote_spanned,
    spanned::Spanned,
    token::Brace,
};

#[proc_macro]
pub fn specialize(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let output = specialize_impl(input.into()).unwrap_or_else(|err| err.into_compile_error());

    eprintln!("{output}");
    output.into()
}

fn specialize_impl(input: TokenStream) -> Result<TokenStream> {
    let internals = internals();

    let Match {
        scrutinee,
        brace,
        arms,
    } = syn::parse2::<Match>(input)?;

    let mut specializations = Vec::new();
    let mut specialization_checks = Vec::new();
    let mut fallback = None;

    for (index, arm) in arms.iter().enumerate() {
        if fallback.is_some() {
            return Err(Error::new(arm.span(), "Wildcard arm must be last"));
        }

        match &arm.ty_match.ty {
            Type::Infer(_) => {
                fallback = Some(&arm.expr);
            }

            _ => {
                let specialization = Specialization::new(index, arm.clone(), scrutinee.clone());
                specialization_checks.push(specialization.check());
                specializations.push(specialization);
            }
        }
    }

    let fallback = fallback.cloned().unwrap_or_else(|| {
        parse_quote_spanned! {brace.span.close()=>
            {
                compile_error!("Missing fallback branch");
                unreachable!()
            }
        }
    });

    Ok(quote! {
        {
            use #internals::InferFallback as _;

            #(#specializations)*

            #(#specialization_checks)*
            { #fallback }
        }
    })
}

fn internals() -> TokenStream {
    let crate_ = match proc_macro_crate::crate_name("specialize") {
        Ok(proc_macro_crate::FoundCrate::Name(name)) => {
            let name = Ident::new(&name, Span::call_site());
            quote! { ::#name }
        }
        _ => quote! { ::specialize },
    };

    quote! {
        #crate_::macro_internals
    }
}

struct Match {
    scrutinee: Scrutinee,
    brace: Brace,
    arms: Vec<MatchArm>,
}

impl Parse for Match {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse::<Token![match]>()?;
        let scrutinee = input.parse()?;

        let content;
        let brace = braced!(content in input);

        let mut arms = Vec::new();
        while !content.is_empty() {
            let arm = content.parse()?;
            arms.push(arm);
        }

        Ok(Self {
            scrutinee,
            brace,
            arms,
        })
    }
}

#[derive(Clone)]
enum Scrutinee {
    Variable(Ident),
    Type(Ident),
}

impl Parse for Scrutinee {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Token![type]) {
            input.parse::<Token![type]>()?;
            input.parse().map(Self::Type)
        } else {
            input.parse().map(Self::Variable)
        }
    }
}

#[derive(Clone)]
struct MatchArm {
    ty_match: TypeMatch,
    expr: syn::Expr,
}

impl MatchArm {
    fn span(&self) -> Span {
        let ty_span = self.ty_match.span();
        let expr_span = self.expr.span();
        ty_span.join(expr_span).unwrap_or(ty_span)
    }
}

impl Parse for MatchArm {
    fn parse(input: ParseStream) -> Result<Self> {
        let ty = input.parse()?;
        input.parse::<Token![=>]>()?;

        let is_block = input.peek(Brace);
        let expr = input.parse()?;

        if !input.is_empty() {
            // trailing comma is optional for blocks but
            // required otherwise
            if is_block {
                input.parse::<Option<Token![,]>>()?;
            } else {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Self { ty_match: ty, expr })
    }
}

#[derive(Clone)]
struct Specialization {
    match_name: Ident,
    nomatch_name: Ident,
    arm: MatchArm,
    scrutinee: Scrutinee,
}

impl Specialization {
    fn new(index: usize, arm: MatchArm, scrutinee: Scrutinee) -> Self {
        let match_name = format_ident!("__Specialization{index}");
        let nomatch_name = format_ident!("__Default{index}");
        Self {
            match_name,
            nomatch_name,
            arm,
            scrutinee,
        }
    }

    fn check(&self) -> SpecializationCheck {
        SpecializationCheck {
            match_name: self.match_name.clone(),
            nomatch_name: self.nomatch_name.clone(),
            arm: self.arm.clone(),
            scrutinee: self.scrutinee.clone(),
        }
    }
}

impl ToTokens for Specialization {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let internals = internals();
        let span = self.arm.span();

        let Self {
            match_name,
            nomatch_name,
            arm:
                MatchArm {
                    ty_match: ty_match @ TypeMatch { ty, default_ty, .. },
                    ..
                },
            ..
        } = self;

        let (impl_generics, impl_ty) = ty_match.split_impl();

        let default_ty = default_ty.clone().unwrap_or_else(|| {
            Type::Never(TypeNever {
                bang_token: Token![!](ty_match.span()),
            })
        });

        tokens.append_all(quote_spanned! {span=>
            struct #match_name;
            struct #nomatch_name;

            #[allow(unused)]
            const _: () = {
                impl #internals::MatchResult for #match_name {
                    const SINGLETON: Option<Self> = Some(Self);
                }
                impl #impl_generics #internals::Specialization<#impl_ty> for #match_name {}

                impl #match_name {
                    const fn cast #impl_generics(&self, value: #impl_ty) -> #ty {
                        value
                    }
                }

                impl #internals::MatchResult for #nomatch_name {
                    const SINGLETON: Option<Self> = None;
                }

                impl #nomatch_name {
                    const fn cast<T>(&self, _value: T) -> #default_ty {
                        unreachable!()
                    }
                }
            };
        });
    }
}

struct SpecializationCheck {
    match_name: Ident,
    nomatch_name: Ident,
    arm: MatchArm,
    scrutinee: Scrutinee,
}

impl ToTokens for SpecializationCheck {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let internals = internals();
        let span = self.arm.span();

        let Self {
            match_name,
            nomatch_name,
            arm:
                MatchArm {
                    ty_match: TypeMatch { generics, ty, .. },
                    expr,
                },
            scrutinee,
        } = self;

        match scrutinee {
            Scrutinee::Variable(scrutinee) => tokens.append_all(quote_spanned! {span=>
                if let Some(m) = #internals::match_specialization_of::<
                    #match_name, #nomatch_name, _, _,
                >(
                    &#scrutinee,
                    |i| i.infer_match(),
                ) {
                    const fn assert_ty #generics(value: #ty) -> #ty { value }

                    #[allow(unused_variables, clippy::diverging_sub_expression)]
                    let #scrutinee = m.cast(#scrutinee);
                    #[allow(unused_variables, unreachable_code)]
                    let #scrutinee = assert_ty(#scrutinee);
                    #expr
                } else
            }),
            Scrutinee::Type(scrutinee_ty) => tokens.append_all(quote_spanned! {span=>
                if #internals::match_specialization::<#match_name, #nomatch_name, #scrutinee_ty, _>(
                    |i| i.infer_match(),
                ).is_some() {
                    #expr
                } else
            }),
        }
    }
}

#[derive(Clone)]
struct TypeMatch {
    generics: Generics,
    ty: Type,
    default_ty: Option<Type>,
}

impl TypeMatch {
    fn split_impl(&self) -> (Generics, Type) {
        match &self.ty {
            Type::ImplTrait(TypeImplTrait { bounds, .. }) => {
                let span = self.ty.span();

                let type_ = Ident::new("__T", span);

                let mut generics = self.generics.clone();

                let insert_pos = generics
                    .params
                    .iter()
                    .position(|p| matches!(p, GenericParam::Lifetime(_)))
                    .unwrap_or(0);

                generics.params.insert(
                    insert_pos,
                    GenericParam::Type(TypeParam {
                        attrs: Vec::new(),
                        ident: type_.clone(),
                        colon_token: Some(Token![:](span)),
                        bounds: bounds.clone(),
                        eq_token: None,
                        default: None,
                    }),
                );

                let type_ = Type::Path(syn::TypePath {
                    qself: None,
                    path: type_.into(),
                });

                (generics, type_)
            }

            _ => (self.generics.clone(), self.ty.clone()),
        }
    }

    fn span(&self) -> Span {
        self.ty.span()
    }
}

impl Parse for TypeMatch {
    fn parse(input: ParseStream) -> Result<Self> {
        let generics = if input.peek(Token![for]) {
            input.parse::<Token![for]>()?;

            let lookahead = input.lookahead1();
            if !lookahead.peek(Token![<]) {
                return Err(lookahead.error());
            }

            input.parse::<Generics>()?
        } else {
            Generics::default()
        };

        let ty = input.parse::<Type>()?;

        let default_ty = if let Type::ImplTrait(_) = &ty
            && input.parse::<Option<Token![default]>>()?.is_some()
        {
            Some(input.parse::<Type>()?)
        } else {
            None
        };

        Ok(Self {
            generics,
            ty,
            default_ty,
        })
    }
}
