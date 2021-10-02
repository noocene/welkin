mod error;

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse_quote;
use syn::punctuated::Punctuated;
use syn::GenericArgument;
use syn::Token;
use syn::WhereClause;
use synstructure::Structure;

pub fn derive(structure: &Structure) -> TokenStream {
    let variant_count = structure.variants().len();

    let (to_welkin_error, to_welkin_error_ident, mut error_variant_idents) =
        error::derive(structure);

    let mut to_welkin = quote!();

    for (idx, variant) in structure.variants().iter().rev().enumerate() {
        let mut stream = quote! {
            let mut term = Term::Variable(Index(#idx));
        };

        for binding in variant.bindings() {
            let ident = &binding.binding;

            let error_variant_ident = error_variant_idents.pop().unwrap();

            stream = quote! {
                #stream

                term = Term::Apply {
                    erased: false,
                    function: Box::new(term),
                    argument: Box::new(ToWelkin::to_welkin(ToAnalogue::to_analogue(#ident)).map_err(#to_welkin_error_ident::#error_variant_ident)?)
                };
            };
        }

        let pat = variant.pat();

        to_welkin = quote! {
            #to_welkin
            #pat => {
                #stream
                term
            }
        };
    }

    to_welkin = quote! {
        match self {
            #to_welkin
        }
    };

    let mut where_clause: WhereClause = parse_quote!(where);

    let mut error_generics: Punctuated<GenericArgument, Token![,]> = parse_quote!();

    for variant in structure.variants() {
        for binding in variant.bindings() {
            let ty = &binding.ast().ty;
            error_generics.push(parse_quote! {
                <<#ty as ToAnalogue>::Analogue as ToWelkin>::Error
            });
            where_clause.predicates.push(parse_quote! {
                #ty: ToAnalogue
            });
        }
    }

    structure.gen_impl(quote! {
        extern crate welkin_binding;
        use welkin_binding::{welkin_core, ToWelkin, ToAnalogue, Error};
        use welkin_core::term::{Term, Index};
        use ::std::{convert::Infallible, boxed::Box, fmt::Debug};

        #to_welkin_error

        gen impl ToWelkin for @Self #where_clause {
            type Error = #to_welkin_error_ident<#error_generics>;

            fn to_welkin(self) -> Result<Term<String>, Self::Error> {
                let mut term = #to_welkin;

                for _ in 0..#variant_count {
                    term = Term::Lambda {
                        erased: false,
                        body: Box::new(term)
                    }
                }

                Ok(term)
            }
        }
    })
}
