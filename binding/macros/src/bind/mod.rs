use std::{collections::HashMap, path::PathBuf};

use anyhow::{anyhow, Error};

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use regex::Regex;
use syn::{
    parenthesized, parse::Parse, parse2, parse_quote, punctuated::Punctuated, File, Ident, Item,
    LitStr, Token, Variant,
};

use welkin_binding_lib::{
    deserialize_defs, welkin_core::term::Term, AbsolutePath, SerializableData,
};

fn error(err: Error) -> TokenStream {
    let err = format!("{}", err);

    quote! {
        const _: () = {
            compile_error!(#err);
            ()
        };
    }
}

pub fn bind(mod_declaration: TokenStream) -> TokenStream {
    match bind_inner(mod_declaration) {
        Ok(data) => data,
        Err(err) => error(err),
    }
}

struct PathArg {
    _eq_token: Token![=],
    path: LitStr,
}

impl Parse for PathArg {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(PathArg {
            _eq_token: input.parse()?,
            path: input.parse()?,
        })
    }
}

struct IdentList {
    _parens: syn::token::Paren,
    list: Punctuated<Ident, Token![,]>,
}

struct OverrideData {}

impl Parse for IdentList {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let list;
        Ok(IdentList {
            _parens: parenthesized!(list in input),
            list: Punctuated::parse_terminated(&list)?,
        })
    }
}

fn bind_inner(mod_declaration: TokenStream) -> Result<TokenStream, Error> {
    let items: File = parse2(mod_declaration)?;

    if !items.shebang.is_none() {
        Err(anyhow!("unexpected shebang"))?;
    }

    if !items.attrs.is_empty() {
        Err(anyhow!(
            "unexpected file-level attribute in bind declaration"
        ))?;
    }

    let mut modules = vec![];

    for item in items.items {
        if let Item::Mod(module) = item {
            modules.push(module)
        } else {
            Err(anyhow!(
                "bind! call should contain only module declarations"
            ))?;
        }
    }

    let mut defs_stream = quote!();

    for module in modules {
        let vis = module.vis;
        let ident = module.ident;

        if let Some((_, items)) = module.content {
            let mut override_enums = HashMap::new();
            let mut override_types = HashMap::new();

            for item in items {
                if let Item::Enum(item) = item {
                    override_enums.insert(format!("{}", item.ident), item);
                } else if let Item::Type(item) = item {
                    override_types.insert(format!("{}", item.ident), item);
                } else {
                    Err(anyhow!(
                        "items in modules should be override enums or aliases"
                    ))?;
                }
            }

            if let Some(attr) = module
                .attrs
                .iter()
                .find(|attr| attr.path == parse_quote!(path))
            {
                let attr: PathArg = parse2(attr.tokens.clone())?;

                let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")?;

                let path = PathBuf::from(manifest_dir);
                let path = path.join(attr.path.value());

                let data = std::fs::read(path)?;

                let mut defs = deserialize_defs(&data)?;

                let mut includes = vec![];

                for attr in module.attrs {
                    if attr.path == parse_quote!(exclude) {
                        let meta: IdentList = parse2(attr.tokens)?;

                        for ident in meta.list {
                            let ident = ident.to_string();
                            if let Some(pos) = defs.iter().position(|def| &def.ident == &ident) {
                                defs.remove(pos);
                            }
                        }
                    } else if attr.path == parse_quote!(include) {
                        let meta: IdentList = parse2(attr.tokens)?;

                        for ident in meta.list {
                            let ident = ident.to_string();
                            includes.push(ident);
                        }
                    }
                }

                let mut override_data = HashMap::new();

                for (name, _) in &override_enums {
                    override_data.insert(name.as_str(), OverrideData {});
                }

                for (name, ty) in &override_types {
                    match &*ty.ty {
                        syn::Type::Path(path) => {
                            if let Some(segment) = path.path.segments.first() {
                                if path.path.segments.len() == 1 {
                                    includes.push(format!("{}", segment.ident));
                                    continue;
                                }
                            }
                            Err(anyhow!("expected unit length type path in alias"))?;
                        }
                        _ => Err(anyhow!("expected type path in alias"))?,
                    }

                    override_data.insert(name.as_str(), OverrideData {});
                }

                let mut override_stream = quote!();

                for (name, en) in &override_enums {
                    let ident = format_ident!("{}", name);
                    let variants = &en.variants;
                    let generics = &en.generics;

                    override_stream = quote! {
                        #override_stream

                        #[allow(non_camel_case_types)]
                        #[derive(Debug, Clone, Adt)]
                        #vis enum #ident #generics {
                            #variants
                        }
                    };
                }

                if !includes.is_empty() {
                    if let Some(include) = includes
                        .iter()
                        .find(|ident| !defs.iter().any(|def| &def.ident == *ident))
                    {
                        return Err(anyhow!("unknown type {}", include));
                    }
                    defs.retain(|def| includes.contains(&def.ident));
                }

                let defs = generate_defs(&defs, &override_data)?;

                defs_stream = quote! {
                    #defs_stream

                    #vis mod #ident {
                        extern crate welkin_binding;

                        use welkin_binding::Adt;

                        #override_stream

                        #defs
                    }
                };
            } else {
                Err(anyhow!("missing `path` attribute"))?;
            }
        } else {
            Err(anyhow!("non-inline modules are currently unsupported"))?;
        }
    }

    Ok(defs_stream)
}

fn generate_defs(
    defs: &[SerializableData],
    overrides: &HashMap<&str, OverrideData>,
) -> Result<TokenStream, Error> {
    let mut defs_stream = quote!();

    for def in defs {
        let ident = format_ident!("{}", def.ident);

        let mut type_arguments: Punctuated<Ident, Token![,]> = parse_quote!();

        for arg in 0..def.type_arguments {
            type_arguments.push(format_ident!("{}", ('A' as u8 + arg as u8) as char));
        }

        let mut variants: Punctuated<Variant, Token![,]> = parse_quote!();

        for (ident, variant) in &def.variants {
            let ident = format_ident!("r#{}", ident);

            let mut fields: Punctuated<TokenStream, Token![,]> = parse_quote!();

            for (ident, field) in &variant.inhabitants {
                let ident = format_ident!("r#{}", ident);

                let (mut ty, is_inductive) = term_to_ty(field, defs, overrides, &def.ident)?;

                if is_inductive {
                    ty = quote!(Box<#ty>);
                }

                let inductive_attr = if is_inductive {
                    quote!(#[inductive])
                } else {
                    quote!()
                };

                fields.push(quote! {
                    #inductive_attr
                    #ident: #ty
                });
            }

            if fields.len() == 0 {
                variants.push(parse_quote! {
                    #ident
                })
            } else {
                variants.push(parse_quote! {
                    #ident {
                        #fields
                    }
                });
            }
        }

        defs_stream = quote! {
            #defs_stream

            #[derive(Debug, Adt, Clone, PartialEq, Hash)]
            #[allow(non_camel_case_types)]
            pub enum #ident<#type_arguments> {
                #variants
            }
        };
    }

    Ok(defs_stream)
}

fn term_to_ty(
    term: &Term<AbsolutePath>,
    defs: &[SerializableData],
    overrides: &HashMap<&str, OverrideData>,
    this: &str,
) -> Result<(TokenStream, bool), Error> {
    let expr = Regex::new("^T[0-9]*$").unwrap();

    Ok(match term {
        Term::Apply {
            function,
            argument,
            erased: true,
        } => {
            let mut arguments = vec![&**argument];
            let mut function = &**function;

            while let Term::Apply {
                function: f,
                argument,
                erased: true,
            } = function
            {
                function = &**f;
                arguments.push(&**argument);
            }

            arguments.reverse();

            let name = if let Term::Reference(path) = function {
                if let Some(ident) = path.0.first() {
                    if path.0.len() == 1 {
                        Some(ident)
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };

            let (function, _) = term_to_ty(function, defs, overrides, this)?;

            let mut args: Punctuated<TokenStream, Token![,]> = parse_quote!();

            if let Some(def) = name.and_then(|name| defs.iter().find(|def| &def.ident == name)) {
                arguments.truncate(arguments.len() - def.indices);
            }

            for argument in arguments {
                args.push(term_to_ty(argument, defs, overrides, this)?.0);
            }

            (
                quote! {
                    #function<#args>
                },
                if let Some(name) = name {
                    name == this
                } else {
                    false
                },
            )
        }
        Term::Reference(reference) => {
            if let Some(segment) = reference.0.first() {
                if reference.0.len() == 1 {
                    if let Some(_) = overrides.get(segment.as_str()) {
                        let ident = format_ident!("r#{}", segment);
                        return Ok((quote!(#ident), false));
                    } else if defs.iter().any(|def| &def.ident == segment) {
                        let ident = format_ident!("r#{}", segment);
                        return Ok((quote!(#ident), false));
                    } else if expr.is_match(segment) {
                        let ident: String = segment.chars().skip(1).collect();
                        let ident: u8 = ident.parse()?;
                        let ident = format_ident!("{}", ('A' as u8 + ident) as char);
                        return Ok((quote!(#ident), false));
                    }
                }
            }
            return Err(anyhow!("unsupported type: {:?}", term));
        }
        _ => Err(anyhow!("unsupported type {:?}", term))?,
    })
}
