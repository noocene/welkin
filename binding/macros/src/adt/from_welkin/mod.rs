mod error;

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse_quote;
use syn::punctuated::Punctuated;
use syn::GenericArgument;
use syn::Token;
use syn::WhereClause;
use synstructure::Structure;

use crate::adt::is_field_inductive;
use crate::adt::is_inductive;

pub fn derive(structure: &Structure) -> TokenStream {
    let variant_count = structure.variants().len();

    let (from_welkin_error, from_welkin_error_ident, mut error_variant_idents) =
        error::derive(structure);

    let mut from_welkin = quote!();

    let mut where_clause: WhereClause = parse_quote!(where);

    let mut error_generics: Punctuated<GenericArgument, Token![,]> = parse_quote!();

    for variant in structure.variants() {
        for binding in variant.bindings() {
            let ty = &binding.ast().ty;

            if !is_inductive(binding) {
                error_generics.push(parse_quote! {
                    <<#ty as FromAnalogue>::Analogue as FromWelkin>::Error
                });
                where_clause.predicates.push(parse_quote! {
                    #ty: FromAnalogue
                });
            }
        }
    }

    for (idx, variant) in structure.variants().iter().rev().enumerate() {
        let construct = variant.construct(|field, _| {
            let mut error_transform = quote! {};

            if !is_field_inductive(field) {
                let error_variant_ident = error_variant_idents.pop().unwrap();

                error_transform = quote! {
                    .map_err(#from_welkin_error_ident::#error_variant_ident)
                };
            }

            quote! {
                FromAnalogue::from_analogue(FromWelkin::from_welkin(fields.next().ok_or(#from_welkin_error_ident::InsufficientFields)?)#error_transform?)
            }
        });
        from_welkin = quote! {
            #from_welkin
            #idx => {
                #construct
            }
        };
    }

    structure.gen_impl(quote! {
        extern crate welkin_binding;
        use welkin_binding::{welkin_core, FromWelkin, FromAnalogue, Error};
        use welkin_core::term::{Term, Index};
        use ::std::{convert::Infallible, boxed::Box, fmt::Debug};

        #from_welkin_error

        gen impl FromWelkin for @Self #where_clause {
            type Error = #from_welkin_error_ident<#error_generics>;

            fn from_welkin(mut term: Term<String>) -> Result<Self, Self::Error> {
                for _ in 0..#variant_count {
                    if let Term::Lambda { body, .. } = term {
                        term = *body;
                    } else {
                        return Err(#from_welkin_error_ident::ExpectedLambda(term))
                    }
                }

                let mut fields = vec![];

                let index = loop {
                    match term {
                        Term::Apply { argument, function, .. } => {
                            term = *function;
                            fields.push(*argument);
                        }
                        Term::Variable(index) => break index.0,
                        other => return Err(#from_welkin_error_ident::ExpectedApplyOrVariable(other))
                    }
                };

                let mut fields = fields.into_iter().rev();

                Ok(match index {
                    #from_welkin
                    index => return Err(#from_welkin_error_ident::InvalidVariant {
                        expected_at_most: #variant_count,
                        got: index
                    })
                })
            }
        }
    })
}
