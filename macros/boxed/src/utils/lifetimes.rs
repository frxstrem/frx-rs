use macro_helpers::error::gather_errors;
use proc_macro2::Span;
use syn::{Result, parse_quote, spanned::Spanned, visit::Visit, visit_mut::VisitMut};

pub fn set_lifetime_span(mut lt: syn::Lifetime, span: Span) -> syn::Lifetime {
    lt.apostrophe = span;
    lt.ident.set_span(span);
    lt
}

pub fn expand_elided_lifetimes(func: &mut syn::ItemFn) -> Result<()> {
    gather_errors(|| {
        // make all input and output lifetimes explicit
        func.attrs.push(parse_quote! {
            #[deny(elided_lifetimes_in_paths)]
        });
        for input in &mut func.sig.inputs {
            ExplicitTypeLifetimes.visit_fn_arg_mut(input);
        }
        ExplicitTypeLifetimes.visit_return_type_mut(&mut func.sig.output);

        // add explicit lifetimes for receiver
        if let Some(syn::FnArg::Receiver(receiver)) = func.sig.inputs.first_mut() {
            let self_lifetime_count = find_lifetimes(|v| v.visit_type(&receiver.ty)).len();

            let mut n = 0;
            ReplaceElidedTypeLifetimes(|lt| {
                *lt = if n == 0 && self_lifetime_count == 1 {
                    syn::Lifetime::new("'__self", lt.span())
                } else {
                    syn::Lifetime::new(&format!("'__self{n}"), lt.span())
                };

                n += 1;
                func.sig.generics.params.insert(
                    0,
                    syn::GenericParam::Lifetime(syn::LifetimeParam::new(lt.clone())),
                );
            })
            .visit_receiver_mut(receiver);
        }

        // add explicit lifetimes for other argument types
        let mut n = 0;
        let mut iv = ReplaceElidedTypeLifetimes(|lt| {
            *lt = syn::Lifetime::new(&format!("'__arg{n}"), lt.span());
            n += 1;
            func.sig.generics.params.insert(
                func.sig
                    .generics
                    .params
                    .iter()
                    .rposition(|param| matches!(param, syn::GenericParam::Lifetime(_)))
                    .map_or(0, |n| n + 1),
                syn::GenericParam::Lifetime(syn::LifetimeParam::new(lt.clone())),
            );
        });

        for input in &mut func.sig.inputs {
            iv.visit_fn_arg_mut(input);
        }

        // add lifetimes for output types
        let receiver_lifetimes = find_lifetimes(|v| {
            if let Some(syn::FnArg::Receiver(receiver)) = func.sig.inputs.first() {
                v.visit_receiver(receiver);
            }
        });

        let arg_lifetimes = find_lifetimes(|v| {
            for input in &func.sig.inputs {
                v.visit_fn_arg(input);
            }
        });

        ReplaceElidedTypeLifetimes(|lt| {
            if receiver_lifetimes.len() == 1 {
                *lt = set_lifetime_span(receiver_lifetimes[0].clone(), lt.span());
            } else if receiver_lifetimes.is_empty() && arg_lifetimes.len() == 1 {
                *lt = set_lifetime_span(arg_lifetimes[0].clone(), lt.span());
            }
        })
        .visit_return_type_mut(&mut func.sig.output);
    })
}

struct ExplicitTypeLifetimes;

impl VisitMut for ExplicitTypeLifetimes {
    fn visit_receiver_mut(&mut self, i: &mut syn::Receiver) {
        syn::visit_mut::visit_type_mut(self, &mut i.ty);

        if let Some((_, ref_lifetime)) = &mut i.reference
            && let syn::Type::Reference(syn::TypeReference { lifetime, .. }) = &mut *i.ty
        {
            *ref_lifetime = lifetime.clone();
        }
    }

    fn visit_type_trait_object_mut(&mut self, i: &mut syn::TypeTraitObject) {
        let has_lifetime = i
            .bounds
            .iter()
            .any(|bound| matches!(bound, syn::TypeParamBound::Lifetime(_)));

        if !has_lifetime {
            i.bounds
                .insert(0, syn::Lifetime::new("'_", i.span()).into());
        }

        syn::visit_mut::visit_type_trait_object_mut(self, i);
    }

    fn visit_type_reference_mut(&mut self, i: &mut syn::TypeReference) {
        if i.lifetime.is_none() {
            i.lifetime = Some(syn::Lifetime::new("'_", i.span()));
        }

        syn::visit_mut::visit_type_reference_mut(self, i);
    }
}

struct ReplaceElidedTypeLifetimes<F: FnMut(&mut syn::Lifetime)>(F);

impl<F: FnMut(&mut syn::Lifetime)> VisitMut for ReplaceElidedTypeLifetimes<F> {
    fn visit_receiver_mut(&mut self, i: &mut syn::Receiver) {
        syn::visit_mut::visit_type_mut(self, &mut i.ty);

        if let Some((_, ref_lifetime)) = &mut i.reference
            && let syn::Type::Reference(syn::TypeReference { lifetime, .. }) = &mut *i.ty
        {
            *ref_lifetime = lifetime.clone();
        }
    }

    fn visit_lifetime_mut(&mut self, i: &mut syn::Lifetime) {
        if i.ident == "_" {
            (self.0)(i);
        }
    }
}

pub fn find_lifetimes<'a>(f: impl FnOnce(&mut dyn Visit<'a>)) -> Vec<&'a syn::Lifetime> {
    #[derive(Default)]
    struct Visitor<'a> {
        lifetimes: Vec<&'a syn::Lifetime>,
    }

    impl<'a> Visit<'a> for Visitor<'a> {
        fn visit_lifetime(&mut self, lt: &'a syn::Lifetime) {
            let exists = lt.ident != "_" && self.lifetimes.iter().any(|it| it.ident == lt.ident);

            if !exists {
                self.lifetimes.push(lt);
            }
        }
    }

    let mut v = Visitor::default();
    f(&mut v);
    v.lifetimes
}
