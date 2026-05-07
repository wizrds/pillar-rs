use heck::ToSnakeCase;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Type;

use crate::attr::FieldAttrs;
use crate::column::column_info;


pub fn resolve_name(ident: &syn::Ident, override_name: Option<&str>) -> String {
    override_name
        .map(str::to_owned)
        .unwrap_or_else(|| ident.to_string().to_snake_case())
}

pub fn columns_body(fields: &[FieldAttrs]) -> syn::Result<TokenStream> {
    let defs = fields
        .iter()
        .filter(|f| !f.skip)
        .map(column_def_tokens)
        .collect::<syn::Result<Vec<_>>>()?;

    let count = defs.len();

    Ok(quote! {
        static COLUMNS: [::pillar::column::ColumnDef; #count] = [#(#defs),*];
        &COLUMNS
    })
}

pub fn companion_module(
    struct_ident: &syn::Ident,
    table_name: &str,
    fields: &[FieldAttrs],
) -> syn::Result<TokenStream> {
    let active_fields = fields.iter().filter(|f| !f.skip).collect::<Vec<_>>();

    let struct_fields = active_fields.iter().map(|f| {
        let ident = f.ident.as_ref().unwrap();
        let ty = &f.ty;
        quote! { pub #ident: #ty }
    }).collect::<Vec<_>>();

    let columns_body = columns_body(fields)?;

    let column_fns = active_fields.iter()
        .map(|f| typed_column_fn(f))
        .collect::<syn::Result<Vec<_>>>()?;

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

        pub struct Column;

        impl Column {
            #(#column_fns)*
        }
    })
}

fn unwrap_option(ty: &Type) -> Option<&Type> {
    let Type::Path(type_path) = ty else { return None };
    let segment = type_path.path.segments.last()?;
    if segment.ident != "Option" { return None }
    let syn::PathArguments::AngleBracketed(args) = &segment.arguments else { return None };
    let syn::GenericArgument::Type(inner) = args.args.first()? else { return None };
    Some(inner)
}

fn typed_column_fn(field: &FieldAttrs) -> syn::Result<TokenStream> {
    let ident = field.ident.as_ref().unwrap();
    let col_name = field.column.as_deref()
        .map(str::to_owned)
        .unwrap_or_else(|| ident.to_string());

    let rust_ty = unwrap_option(&field.ty).unwrap_or(&field.ty);

    Ok(quote! {
        pub fn #ident() -> ::pillar::column::TypedColumn<#rust_ty> {
            ::pillar::column::TypedColumn::new(#col_name)
        }
    })
}

fn column_def_tokens(field: &FieldAttrs) -> syn::Result<TokenStream> {
    let name = field.column.as_deref()
        .map(str::to_owned)
        .unwrap_or_else(|| field.ident.as_ref().unwrap().to_string());

    let info = column_info(&field.ty)?;
    let column_type = info.column_type;
    let nullable = info.nullable;
    let primary_key = field.primary_key;
    let unique = field.unique;

    Ok(quote! {
        ::pillar::column::ColumnDef {
            name: #name,
            column_type: #column_type,
            nullable: #nullable,
            primary_key: #primary_key,
            unique: #unique,
        }
    })
}
