use proc_macro2::TokenStream;
use quote::quote;

use crate::attr::FieldAttrs;


pub fn columns_body(fields: &[FieldAttrs]) -> syn::Result<TokenStream> {
    let defs = fields
        .iter()
        .filter(|f| !f.skip)
        .map(column_def)
        .collect::<syn::Result<Vec<_>>>()?;

    let count = defs.len();

    Ok(quote! {
        static COLUMNS: [::pillar::column::ColumnDef; #count] = [#(#defs),*];
        &COLUMNS
    })
}

pub fn column_struct(fields: &[FieldAttrs]) -> syn::Result<TokenStream> {
    let fns = fields
        .iter()
        .filter(|f| !f.skip)
        .map(typed_column_fn)
        .collect::<syn::Result<Vec<_>>>()?;

    Ok(quote! {
        pub struct Column;

        impl Column {
            #(#fns)*
        }
    })
}

fn column_def(field: &FieldAttrs) -> syn::Result<TokenStream> {
    let name = field.column.as_deref()
        .map(str::to_owned)
        .unwrap_or_else(|| field.ident.as_ref().unwrap().to_string());

    let info = if let Some(raw) = &field.column_type {
        ColumnInfo {
            column_type: quote! { ::pillar::ast::ColumnType::Custom(#raw.to_string()) },
            nullable: unwrap_option(&field.ty).is_some(),
        }
    } else {
        column_info(&field.ty)?
    };
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

pub struct ColumnInfo {
    pub column_type: TokenStream,
    pub nullable: bool,
}

pub fn column_info(ty: &syn::Type) -> syn::Result<ColumnInfo> {
    if let Some(inner) = unwrap_option(ty) {
        return Ok(ColumnInfo {
            column_type: type_to_column_type(inner)?,
            nullable: true,
        });
    }

    Ok(ColumnInfo {
        column_type: type_to_column_type(ty)?,
        nullable: false,
    })
}

fn unwrap_option(ty: &syn::Type) -> Option<&syn::Type> {
    let syn::Type::Path(type_path) = ty else { return None };
    let segment = type_path.path.segments.last()?;

    if segment.ident != "Option" { return None }

    let syn::PathArguments::AngleBracketed(args) = &segment.arguments else { return None };
    let syn::GenericArgument::Type(inner) = args.args.first()? else { return None };

    Some(inner)
}

fn type_to_column_type(ty: &syn::Type) -> syn::Result<TokenStream> {
    let syn::Type::Path(type_path) = ty else {
        return Err(syn::Error::new_spanned(ty, "unsupported field type"));
    };

    path_to_column_type(&type_path.path)
}

fn path_to_column_type(path: &syn::Path) -> syn::Result<TokenStream> {
    let segment = path.segments.last().unwrap();

    match segment.ident.to_string().as_str() {
        "bool" => Ok(quote! { ::pillar::ast::ColumnType::Boolean }),
        "i8" => Ok(quote! { ::pillar::ast::ColumnType::Int8 }),
        "i16" => Ok(quote! { ::pillar::ast::ColumnType::Int16 }),
        "i32" => Ok(quote! { ::pillar::ast::ColumnType::Int32 }),
        "i64" => Ok(quote! { ::pillar::ast::ColumnType::Int64 }),
        "u8" => Ok(quote! { ::pillar::ast::ColumnType::UInt8 }),
        "u16" => Ok(quote! { ::pillar::ast::ColumnType::UInt16 }),
        "u32" => Ok(quote! { ::pillar::ast::ColumnType::UInt32 }),
        "u64" => Ok(quote! { ::pillar::ast::ColumnType::UInt64 }),
        "f32" => Ok(quote! { ::pillar::ast::ColumnType::Float32 }),
        "f64" => Ok(quote! { ::pillar::ast::ColumnType::Float64 }),
        "String" => Ok(quote! { ::pillar::ast::ColumnType::String }),
        "Vec" => vec_to_column_type(path),
        "NaiveDate" => Ok(quote! { ::pillar::ast::ColumnType::Date }),
        "NaiveTime" => Ok(quote! { ::pillar::ast::ColumnType::Time }),
        "DateTime" => Ok(quote! { ::pillar::ast::ColumnType::DateTime }),
        "Uuid" => Ok(quote! { ::pillar::ast::ColumnType::Uuid }),
        _ => Err(syn::Error::new_spanned(
            path,
            format!(
                "unsupported column type `{}`; implement Model manually if needed",
                segment.ident,
            ),
        )),
    }
}

fn vec_to_column_type(path: &syn::Path) -> syn::Result<TokenStream> {
    let syn::PathArguments::AngleBracketed(args) = &path.segments.last().unwrap().arguments else {
        return Err(syn::Error::new_spanned(path, "Vec requires a type argument"));
    };

    match args.args.first() {
        Some(syn::GenericArgument::Type(syn::Type::Path(inner)))
            if inner.path.is_ident("u8") =>
        {
            Ok(quote! { ::pillar::ast::ColumnType::Binary })
        }
        _ => Err(syn::Error::new_spanned(
            path,
            "only Vec<u8> is supported as a column type (Binary)",
        )),
    }
}
