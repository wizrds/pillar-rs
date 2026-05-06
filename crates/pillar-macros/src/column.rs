use proc_macro2::TokenStream;
use quote::quote;


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
        "bool"      => Ok(quote! { ::pillar::column::ColumnType::Boolean }),
        "i8"        => Ok(quote! { ::pillar::column::ColumnType::Int8 }),
        "i16"       => Ok(quote! { ::pillar::column::ColumnType::Int16 }),
        "i32"       => Ok(quote! { ::pillar::column::ColumnType::Int32 }),
        "i64"       => Ok(quote! { ::pillar::column::ColumnType::Int64 }),
        "u8"        => Ok(quote! { ::pillar::column::ColumnType::UInt8 }),
        "u16"       => Ok(quote! { ::pillar::column::ColumnType::UInt16 }),
        "u32"       => Ok(quote! { ::pillar::column::ColumnType::UInt32 }),
        "u64"       => Ok(quote! { ::pillar::column::ColumnType::UInt64 }),
        "f32"       => Ok(quote! { ::pillar::column::ColumnType::Float32 }),
        "f64"       => Ok(quote! { ::pillar::column::ColumnType::Float64 }),
        "String"    => Ok(quote! { ::pillar::column::ColumnType::String }),
        "Vec"       => vec_to_column_type(path),
        "NaiveDate" => Ok(quote! { ::pillar::column::ColumnType::Date }),
        "NaiveTime" => Ok(quote! { ::pillar::column::ColumnType::Time }),
        "DateTime"  => Ok(quote! { ::pillar::column::ColumnType::DateTime }),
        "Uuid"      => Ok(quote! { ::pillar::column::ColumnType::Uuid }),
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
            Ok(quote! { ::pillar::column::ColumnType::Binary })
        }
        _ => Err(syn::Error::new_spanned(
            path,
            "only Vec<u8> is supported as a column type (Binary)",
        )),
    }
}
