use darling::FromDeriveInput;
use proc_macro2::TokenStream;
use syn::DeriveInput;

use crate::attr::ModelAttrs;
use crate::codegen::{companion_module, resolve_name};


pub fn derive(input: DeriveInput) -> TokenStream {
    match ModelAttrs::from_derive_input(&input) {
        Err(e) => e.write_errors(),
        Ok(attrs) => impl_model(attrs).unwrap_or_else(|e| e.into_compile_error()),
    }
}

fn impl_model(attrs: ModelAttrs) -> syn::Result<TokenStream> {
    let ident = &attrs.ident;
    let table_name = resolve_name(ident, attrs.table.as_deref());
    let fields = attrs.data.take_struct().unwrap().fields;

    companion_module(ident, &table_name, &fields)
}
