use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_quote, punctuated::Punctuated, Expr, GenericParam, Token, Type, WhereClause};
use synstructure::Structure;

use crate::adt::is_inductive;

fn substitute_in_type(ty: &mut Type, param_names: &Vec<String>) {
    match ty {
        Type::Path(path) => {
            if let Some(segment) = path.path.segments.first() {
                if let Some(idx) = param_names
                    .iter()
                    .position(|item| item == &format!("{}", segment.ident))
                {
                    *ty = parse_quote! {
                        Dummy<{#idx}>
                    };
                    return;
                }
            }
            for segment in &mut path.path.segments {
                match &mut segment.arguments {
                    syn::PathArguments::AngleBracketed(args) => {
                        for arg in &mut args.args {
                            match arg {
                                syn::GenericArgument::Type(ty) => {
                                    substitute_in_type(ty, param_names);
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
}

pub fn derive(structure: &Structure) -> TokenStream {
    let name = format!("{}", structure.ast().ident);
    let num_params = structure.ast().generics.params.len();
    let mut where_clause: WhereClause = parse_quote!(where);

    let mut params: Punctuated<Expr, Token![,]> = parse_quote!();

    let mut param_names = vec![];

    for param in &structure.ast().generics.params {
        if let GenericParam::Type(param) = param {
            let ident = &param.ident;

            param_names.push(format!("{}", ident));

            where_clause.predicates.push(parse_quote! {
                #ident: Typed
            });

            params.push(parse_quote! {
                <#ident as Typed>::TYPE
            });
        }
    }

    let mut variants: Punctuated<Expr, Token![,]> = parse_quote!();

    for variant in structure.variants() {
        let name = format!("{}", variant.ast().ident);

        let mut fields: Punctuated<Expr, Token![,]> = parse_quote!();

        for binding in variant.bindings() {
            if let Type::Path(path) = &binding.ast().ty {
                let path = &path.path;
                if let Some(segment) = path.segments.first() {
                    if path.segments.len() == 1 {
                        if let Some(idx) = param_names
                            .iter()
                            .position(|item| item == &format!("{}", segment.ident))
                        {
                            fields.push(parse_quote! {
                                Type::Parameter(#idx)
                            });

                            continue;
                        }
                    }
                }
            }

            let mut ty = binding.ast().ty.clone();

            substitute_in_type(&mut ty, &param_names);

            let constructor = if is_inductive(binding) {
                quote! {
                    AdtConstructor::Inductive
                }
            } else {
                quote! {
                    AdtConstructor::Other(&<<#ty as Analogous>::Analogue as Adt>::DEFINITION)
                }
            };

            fields.push(parse_quote! {
                Type::Data {
                    constructor: #constructor,
                    params: <<#ty as Analogous>::Analogue as Adt>::PARAMS
                }
            })
        }

        variants.push(parse_quote! {
            AdtVariant {
                name: #name,
                fields: &[
                    #fields
                ]
            }
        });
    }

    structure.gen_impl(quote! {
        extern crate welkin_binding;
        use std::marker::PhantomData;
        use welkin_binding::{Adt, Type, Dummy, Typed, AdtDefinition, AdtVariant, AdtConstructor, Analogous};

        gen impl Adt for @Self #where_clause {
            const PARAMS: &'static [Type] = &[
                #params
            ];
            const DEFINITION: AdtDefinition = AdtDefinition {
                name: #name,
                params: #num_params,
                variants: &[
                    #variants
                ],
            };
        }
    })
}
