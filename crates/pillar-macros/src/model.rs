use darling::FromDeriveInput;
use heck::ToSnakeCase;
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

use crate::attr::{FieldAttrs, ModelAttrs};
use crate::column::{column_struct, columns_body};


pub fn derive(input: DeriveInput) -> TokenStream {
    match ModelAttrs::from_derive_input(&input) {
        Err(e) => e.write_errors(),
        Ok(attrs) => impl_model(attrs).unwrap_or_else(|e| e.into_compile_error()),
    }
}

fn impl_model(attrs: ModelAttrs) -> syn::Result<TokenStream> {
    let ident = &attrs.ident;
    let table_name = attrs.table
        .as_deref()
        .map(str::to_owned)
        .unwrap_or_else(|| ident.to_string().to_snake_case());

    let fields = attrs.data.take_struct().unwrap().fields;

    let model = model_struct(&table_name, &fields)?;
    let columns = column_struct(&fields)?;

    Ok(quote! {
        #model
        #columns
    })
}

fn model_struct(table_name: &str, fields: &[FieldAttrs]) -> syn::Result<TokenStream> {
    let struct_fields = fields.iter().filter(|f| !f.skip).map(|f| {
        let ident = f.ident.as_ref().unwrap();
        let ty = &f.ty;
        quote! { pub #ident: #ty }
    });

    let columns_body = columns_body(fields)?;

    Ok(quote! {
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
        pub struct Model {
            #(#struct_fields),*
        }

        impl ::pillar::model::Model for Model {
            fn table_name() -> &'static str {
                #table_name
            }

            fn columns() -> &'static [::pillar::column::ColumnDef] {
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
                        as ::pillar::__private::serde_arrow::schema::SchemaLike>::from_samples(
                        &rows.iter().collect::<::std::vec::Vec<_>>(),
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
