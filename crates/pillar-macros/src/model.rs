use darling::FromDeriveInput;
use heck::ToSnakeCase;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::DeriveInput;

use crate::attr::{FieldAttrs, ModelAttrs, TtlAttr, TtlUnit};
use crate::column::{column_info, column_struct, columns_body};


pub fn derive(input: DeriveInput) -> TokenStream {
    match ModelAttrs::from_derive_input(&input) {
        Err(e) => e.write_errors(),
        Ok(attrs) => impl_model(attrs).unwrap_or_else(|e| e.into_compile_error()),
    }
}

fn impl_model(attrs: ModelAttrs) -> syn::Result<TokenStream> {
    let table_name = attrs.table
        .as_deref()
        .map(str::to_owned)
        .unwrap_or_else(|| attrs.ident.to_string().to_snake_case());

    let fields = attrs.data.clone().take_struct().unwrap().fields;

    let model = model_struct(&table_name, &fields)?;
    let columns = column_struct(&fields)?;
    let table_schema = table_schema_impl(&table_name, &fields, &attrs)?;

    Ok(quote! {
        #model
        #columns
        #table_schema
    })
}

fn model_struct(table_name: &str, fields: &[FieldAttrs]) -> syn::Result<TokenStream> {
    let struct_fields = fields.iter().filter(|f| !f.skip).map(|f| {
        let ident = f.ident.as_ref().unwrap();
        let ty = &f.ty;
        quote! { pub #ident: #ty }
    });

    let columns_body = columns_body(fields)?;

    let row_fields = fields.iter().filter(|f| !f.skip).map(|f| {
        let ident = f.ident.as_ref().unwrap();
        quote! { ::pillar::value::Value::from(self.#ident.clone()) }
    });

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

            fn to_row(&self) -> ::std::vec::Vec<::pillar::value::Value> {
                vec![#(#row_fields),*]
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
                    &Self::columns()
                        .iter()
                        .map(|col| col.to_arrow_field())
                        .collect::<::std::result::Result<::std::vec::Vec<_>, _>>()?,
                    &rows.iter().collect::<::std::vec::Vec<_>>(),
                )
                .map_err(|e| ::pillar::errors::Error::serialization(e.to_string()))
            }
        }
    })
}

fn table_schema_impl(table_name: &str, fields: &[FieldAttrs], attrs: &ModelAttrs) -> syn::Result<TokenStream> {
    let has_ddl = attrs.engine.is_some()
        || attrs.partition_by.is_some()
        || attrs.ttl.is_some()
        || attrs.options.as_ref().map(|o| !o.is_empty()).unwrap_or(false)
        || fields.iter().any(|f| f.order_by);

    if !has_ddl {
        return Ok(quote! {});
    }

    let col_defs = fields.iter()
        .filter(|f| !f.skip)
        .map(|f| {
            let col_name = f.column.as_deref()
                .map(str::to_owned)
                .unwrap_or_else(|| f.ident.as_ref().unwrap().to_string());

            let (col_type, nullable) = if let Some(raw) = &f.column_type {
                (
                    quote! { ::pillar::ast::ColumnType::Custom(#raw.to_string()) },
                    column_info(&f.ty).map(|i| i.nullable).unwrap_or(false),
                )
            } else {
                let info = column_info(&f.ty)?;
                (info.column_type, info.nullable)
            };

            let mut expr = quote! { ::pillar::ast::ColumnDefinition::new(#col_name, #col_type) };

            if nullable {
                expr = quote! { #expr.nullable() };
            }

            if f.primary_key {
                expr = quote! { #expr.primary_key() };
            }

            Ok(expr)
        })
        .collect::<syn::Result<Vec<_>>>()?;

    let order_by_fields: Vec<String> = fields.iter()
        .filter(|f| !f.skip && f.order_by)
        .map(|f| {
            f.column.as_deref()
                .map(str::to_owned)
                .unwrap_or_else(|| f.ident.as_ref().unwrap().to_string())
        })
        .collect();

    let mut option_calls = quote! {};

    if let Some(engine) = &attrs.engine {
        option_calls = quote! { #option_calls .option("engine", #engine) };
    }

    if let Some(partition_by) = &attrs.partition_by {
        option_calls = quote! { #option_calls .option("partition_by", #partition_by) };
    }

    if !order_by_fields.is_empty() {
        let s = format!("({})", order_by_fields.join(", "));
        option_calls = quote! { #option_calls .option("order_by", #s) };
    }

    if let Some(opts) = &attrs.options {
        let mut sorted: Vec<_> = opts.iter().collect();
        sorted.sort_by_key(|(k, _)| k.as_str());
        for (k, v) in sorted {
            option_calls = quote! { #option_calls .option(#k, #v) };
        }
    }

    let ttl_call = match &attrs.ttl {
        Some(ttl) => build_ttl(ttl)?,
        None => quote! {},
    };

    Ok(quote! {
        impl ::pillar::query::TableSchema for Model {
            fn create_statement() -> ::pillar::ast::Statement {
                ::pillar::ast::Statement::CreateTable(
                    ::pillar::ast::CreateTableStatement::new(#table_name)
                        .if_not_exists()
                        .columns(vec![#(#col_defs),*])
                        #option_calls
                        #ttl_call
                )
            }
        }
    })
}

fn build_ttl(ttl: &TtlAttr) -> syn::Result<TokenStream> {
    let unit = ttl_unit_ident(&ttl.unit);
    let column = &ttl.column;
    let interval = ttl.interval;

    Ok(quote! {
        .ttl(::pillar::ast::TtlClause::delete(
            #column,
            ::pillar::ast::Interval::new(#interval, ::pillar::ast::IntervalUnit::#unit),
        ))
    })
}

fn ttl_unit_ident(unit: &TtlUnit) -> syn::Ident {
    let variant = match unit {
        TtlUnit::Second => "Second",
        TtlUnit::Minute => "Minute",
        TtlUnit::Hour => "Hour",
        TtlUnit::Day => "Day",
        TtlUnit::Week => "Week",
        TtlUnit::Month => "Month",
        TtlUnit::Year => "Year",
    };

    syn::Ident::new(variant, Span::call_site())
}
