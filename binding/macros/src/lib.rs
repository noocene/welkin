use synstructure::decl_derive;
mod adt;

decl_derive!([Adt] => adt_derive);

fn adt_derive(item: synstructure::Structure) -> proc_macro2::TokenStream {
    adt::derive(item)
}
