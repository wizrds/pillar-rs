use darling::FromDeriveInput;
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

use crate::attr::ViewAttrs;
use crate::codegen::{columns_body, resolve_name};


pub fn derive(input: DeriveInput) -> TokenStream {
    match ViewAttrs::from_derive_input(&input) {
        Err(e) => e.write_errors(),
        Ok(attrs) => impl_view(attrs).unwrap_or_else(|e| e.into_compile_error()),
    }
}

fn impl_view(attrs: ViewAttrs) -> syn::Result<TokenStream> {
    let ident = &attrs.ident;
    let (impl_generics, ty_generics, where_clause) = attrs.generics.split_for_impl();
    let view_name = resolve_name(&attrs.ident, attrs.view.as_deref());
    let columns_body = columns_body(&attrs.data.take_struct().unwrap().fields)?;

    Ok(quote! {
        impl #impl_generics ::pillar::traits::MaterializedView for #ident #ty_generics #where_clause {
            fn view_name() -> &'static str {
                #view_name
            }

            fn columns() -> &'static [::pillar::traits::ColumnDef] {
                #columns_body
            }

            fn from_record_batch(
                batch: ::pillar::__private::arrow::record_batch::RecordBatch,
            ) -> ::std::result::Result<::std::vec::Vec<Self>, ::pillar::errors::Error> {
                ::pillar::__private::serde_arrow::from_record_batch(&batch)
                    .map_err(|e| ::pillar::errors::Error::serialization(e.to_string()))
            }
        }
    })
}
