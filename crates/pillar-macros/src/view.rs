use darling::FromDeriveInput;
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

use crate::attr::{FieldAttrs, ViewAttrs};
use crate::codegen::{columns_body, resolve_name};
use crate::condition::parse_condition;


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
    let fields = attrs.data.take_struct().unwrap().fields;
    let columns_body = columns_body(&fields)?;

    let materialized_view_impl = quote! {
        impl #impl_generics ::pillar::view::MaterializedView for #ident #ty_generics #where_clause {
            fn view_name() -> &'static str {
                #view_name
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
        }
    };

    let view_query_impl = match attrs.from {
        Some(from_table) => {
            let projections = view_projections(&fields);
            let where_tokens = match attrs.filter {
                Some(ref s) => {
                    let cond = parse_condition(s)?;
                    quote! { .where_clause(#cond) }
                }
                None => quote! {},
            };

            Some(quote! {
                impl #impl_generics ::pillar::view::ViewQuery for #ident #ty_generics #where_clause {
                    fn query() -> ::pillar::ast::SelectStatement {
                        ::pillar::ast::SelectStatement::new(
                            ::pillar::ast::TableRef::new(#from_table)
                        )
                        .projections(vec![#projections])
                        #where_tokens
                    }
                }
            })
        }
        None => None,
    };

    Ok(quote! {
        #materialized_view_impl
        #view_query_impl
    })
}

fn view_projections(fields: &[FieldAttrs]) -> TokenStream {
    let cols = fields
        .iter()
        .filter(|f| !f.skip)
        .map(|f| {
            let name = f.column.as_deref()
                .map(str::to_owned)
                .unwrap_or_else(|| f.ident.as_ref().unwrap().to_string());
            quote! { ::pillar::ast::Projection::Column(#name.to_string()) }
        });

    quote! { #(#cols),* }
}
