use darling::FromDeriveInput;
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

use crate::attr::ModelAttrs;
use crate::codegen::{columns_body, resolve_name};


pub fn derive(input: DeriveInput) -> TokenStream {
    match ModelAttrs::from_derive_input(&input) {
        Err(e) => e.write_errors(),
        Ok(attrs) => impl_model(attrs).unwrap_or_else(|e| e.into_compile_error()),
    }
}

fn impl_model(attrs: ModelAttrs) -> syn::Result<TokenStream> {
    let ident = &attrs.ident;
    let (impl_generics, ty_generics, where_clause) = attrs.generics.split_for_impl();
    let table_name = resolve_name(&attrs.ident, attrs.table.as_deref());
    let columns_body = columns_body(&attrs.data.take_struct().unwrap().fields)?;

    Ok(quote! {
        impl #impl_generics ::pillar::traits::Model for #ident #ty_generics #where_clause {
            fn table_name() -> &'static str {
                #table_name
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

            fn to_record_batch(
                rows: &[Self],
            ) -> ::std::result::Result<::pillar::__private::arrow::record_batch::RecordBatch, ::pillar::errors::Error> {
                ::pillar::__private::serde_arrow::to_record_batch(
                    &<::std::vec::Vec<::pillar::__private::arrow::datatypes::FieldRef>
                        as ::pillar::__private::serde_arrow::schema::SchemaLike>::from_type::<Self>(
                        ::pillar::__private::serde_arrow::schema::TracingOptions::default(),
                    )
                    .map_err(|e| ::pillar::errors::Error::serialization(e.to_string()))?,
                    &rows.iter().collect::<::std::vec::Vec<_>>(),
                )
                .map_err(|e| ::pillar::errors::Error::serialization(e.to_string()))
            }
        }
    })
}
