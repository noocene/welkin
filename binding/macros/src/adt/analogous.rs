use proc_macro2::TokenStream;
use quote::quote;
use synstructure::Structure;

pub fn derive(structure: &Structure) -> TokenStream {
    structure.gen_impl(quote! {
        extern crate welkin_binding;

        gen impl welkin_binding::Analogous for @Self {
            type Analogue = Self;
        }
    })
}
