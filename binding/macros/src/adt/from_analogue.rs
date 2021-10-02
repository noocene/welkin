use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_quote, punctuated::Punctuated, Ident, Token, WhereClause};
use synstructure::Structure;

use crate::adt::is_inductive;

pub fn derive(structure: &Structure) -> TokenStream {
    let mut where_clause: WhereClause = parse_quote!(where);

    let mut inductive_impls = quote! {};

    for variant in structure.variants() {
        for binding in variant.bindings() {
            let ty = &binding.ast().ty;

            if !is_inductive(binding) {
                where_clause.predicates.push(parse_quote! {
                    #ty: FromAnalogue
                });
            } else {
                let mut where_clause: WhereClause = parse_quote!(where);

                let generics: Punctuated<Ident, Token![,]> = binding
                    .referenced_ty_params()
                    .into_iter()
                    .cloned()
                    .collect();

                for parameter in binding.referenced_ty_params() {
                    where_clause.predicates.push(parse_quote! {
                        #parameter: FromAnalogue
                    });
                }

                inductive_impls = quote! {
                    #inductive_impls

                    impl<#generics> FromAnalogue for #ty #where_clause {
                        type Analogue = <<Self as Wrapper>::Inner as FromAnalogue>::Analogue;

                        fn from_analogue(analogue: <Self as FromAnalogue>::Analogue) -> Self {
                            Box::new(FromAnalogue::from_analogue(analogue))
                        }
                    }
                };
            }
        }
    }

    let inductive_uses = if inductive_impls.is_empty() {
        quote! {}
    } else {
        quote! {
            use ::std::boxed::Box;
            use welkin_binding::Wrapper;
        }
    };

    structure.gen_impl(quote! {
        extern crate welkin_binding;
        use welkin_binding::FromAnalogue;

        #inductive_uses

        #inductive_impls

        gen impl FromAnalogue for @Self #where_clause {
            type Analogue = Self;

            fn from_analogue(data: Self) -> Self {
                data
            }
        }
    })
}
