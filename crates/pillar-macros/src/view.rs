use darling::FromDeriveInput;
use heck::ToSnakeCase;
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

use crate::attr::{FieldAttrs, ViewAttrs};
use crate::column::{column_struct, columns_body};
use crate::condition::parse_condition;


pub fn derive(input: DeriveInput) -> TokenStream {
    match ViewAttrs::from_derive_input(&input) {
        Err(e) => e.write_errors(),
        Ok(attrs) => impl_view(attrs).unwrap_or_else(|e| e.into_compile_error()),
    }
}

fn impl_view(attrs: ViewAttrs) -> syn::Result<TokenStream> {
    let ident = &attrs.ident;
    let view_name = attrs.view
        .as_deref()
        .map(str::to_owned)
        .unwrap_or_else(|| ident.to_string().to_snake_case());

    let fields = attrs.data.take_struct().unwrap().fields;

    let view = view_struct(&view_name, &fields, attrs.from.as_deref(), attrs.filter.as_deref())?;
    let columns = column_struct(&fields)?;

    Ok(quote! {
        #view
        #columns
    })
}

fn view_struct(
    view_name: &str,
    fields: &[FieldAttrs],
    from: Option<&str>,
    filter: Option<&str>,
) -> syn::Result<TokenStream> {
    let struct_fields = fields.iter().filter(|f| !f.skip).map(|f| {
        let ident = f.ident.as_ref().unwrap();
        let ty = &f.ty;
        quote! { pub #ident: #ty }
    });

    let columns_body = columns_body(fields)?;

    let view_query_impl = match from {
        Some(from_table) => {
            let projections = fields.iter().filter(|f| !f.skip).map(|f| {
                let name = f.column.as_deref()
                    .map(str::to_owned)
                    .unwrap_or_else(|| f.ident.as_ref().unwrap().to_string());
                quote! { ::pillar::ast::Projection::Column(#name.to_string()) }
            });

            let where_tokens = match filter {
                Some(s) => {
                    let cond = parse_condition(s)?;
                    quote! { .where_clause(#cond) }
                }
                None => quote! {},
            };

            quote! {
                impl ::pillar::view::ViewQuery for View {
                    fn query() -> ::pillar::ast::SelectStatement {
                        ::pillar::ast::SelectStatement::new(
                            ::pillar::ast::TableRef::new(#from_table)
                        )
                        .projections(vec![#(#projections),*])
                        #where_tokens
                    }
                }
            }
        }
        None => quote! {},
    };

    Ok(quote! {
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
        pub struct View {
            #(#struct_fields),*
        }

        impl ::pillar::view::MaterializedView for View {
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

        #view_query_impl
    })
}
