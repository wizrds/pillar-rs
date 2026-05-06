use heck::ToSnakeCase;
use proc_macro2::TokenStream;
use quote::quote;

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
        static COLUMNS: [::pillar::traits::ColumnDef; #count] = [#(#defs),*];
        &COLUMNS
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
        ::pillar::traits::ColumnDef {
            name: #name,
            column_type: #column_type,
            nullable: #nullable,
            primary_key: #primary_key,
            unique: #unique,
        }
    })
}
