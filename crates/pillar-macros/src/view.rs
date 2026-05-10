use darling::FromDeriveInput;
use heck::ToSnakeCase;
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

use crate::attr::{AggregateOp, FieldAttrs, ViewAttrs};
use crate::column::{column_struct, columns_body};
use crate::condition::parse_condition;
use crate::from_batch;


pub fn derive(input: DeriveInput) -> TokenStream {
    match ViewAttrs::from_derive_input(&input) {
        Err(e) => e.write_errors(),
        Ok(attrs) => impl_view(attrs).unwrap_or_else(|e| e.into_compile_error()),
    }
}

fn impl_view(attrs: ViewAttrs) -> syn::Result<TokenStream> {
    let view_name = attrs.view
        .as_deref()
        .map(str::to_owned)
        .unwrap_or_else(|| attrs.ident.to_string().to_snake_case());

    let fields = attrs.data.clone().take_struct().unwrap().fields;

    let view = view_struct(&view_name, &fields, &attrs)?;
    let columns = column_struct(&fields)?;
    let view_schema = view_schema_impl(&fields, &attrs)?;

    Ok(quote! {
        #view
        #columns
        #view_schema
    })
}

fn view_struct(view_name: &str, fields: &[FieldAttrs], attrs: &ViewAttrs) -> syn::Result<TokenStream> {
    let struct_fields = fields.iter().filter(|f| !f.skip).map(|f| {
        let ident = f.ident.as_ref().unwrap();
        let ty = &f.ty;
        quote! { pub #ident: #ty }
    });

    let columns_body = columns_body(fields)?;
    let view_query_impl = view_query_impl(fields, attrs)?;

    let view_ident = syn::Ident::new("View", proc_macro2::Span::call_site());
    let from_batch_impl = from_batch::derive_for(&view_ident);

    Ok(quote! {
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
        pub struct View {
            #(#struct_fields),*
        }

        #from_batch_impl

        impl ::pillar::view::View for View {
            fn view_name() -> &'static str {
                #view_name
            }

            fn columns() -> &'static [::pillar::column::ColumnDef] {
                #columns_body
            }

            fn from_record_batch(
                batch: ::pillar::__private::arrow::record_batch::RecordBatch,
            ) -> ::std::result::Result<::std::vec::Vec<Self>, ::pillar::errors::Error> {
                <Self as ::pillar::convert::FromBatch>::from_batch(batch)
            }
        }

        #view_query_impl
    })
}

fn view_query_impl(fields: &[FieldAttrs], attrs: &ViewAttrs) -> syn::Result<TokenStream> {
    let from_table = match attrs.from.as_deref() {
        Some(t) => t,
        None => return Ok(quote! {}),
    };

    let has_aggregates = fields.iter().any(|f| !f.skip && f.aggregate.is_some());

    let projections = fields.iter().filter(|f| !f.skip).map(|f| {
        let col_name = f.column.as_deref()
            .map(str::to_owned)
            .unwrap_or_else(|| f.ident.as_ref().unwrap().to_string());

        match &f.aggregate {
            None => quote! { ::pillar::ast::Projection::Column(#col_name.to_string()) },
            Some(op) => {
                let source = f.source.as_deref().unwrap_or(&col_name);
                aggregate_projection(op, source)
            }
        }
    });

    let group_by = if has_aggregates {
        let keys = fields.iter().filter(|f| !f.skip && f.aggregate.is_none()).map(|f| {
            let name = f.column.as_deref()
                .map(str::to_owned)
                .unwrap_or_else(|| f.ident.as_ref().unwrap().to_string());
            quote! { #name.to_string() }
        });
        quote! { .group_by(vec![#(#keys),*]) }
    } else {
        quote! {}
    };

    let where_tokens = match attrs.filter.as_deref() {
        Some(s) => {
            let cond = parse_condition(s)?;
            quote! { .where_clause(#cond) }
        }
        None => quote! {},
    };

    Ok(quote! {
        impl ::pillar::view::ViewQuery for View {
            fn query() -> ::pillar::ast::SelectStatement {
                ::pillar::ast::SelectStatement::new(
                    ::pillar::ast::TableRef::new(#from_table)
                )
                .projections(vec![#(#projections),*])
                #group_by
                #where_tokens
            }
        }
    })
}

fn aggregate_projection(op: &AggregateOp, source: &str) -> TokenStream {
    match op {
        AggregateOp::Count => quote! {
            ::pillar::ast::Projection::Aggregate(
                ::pillar::ast::AggregateFunction::Count(::pillar::ast::CountArg::All)
            )
        },
        AggregateOp::Sum => quote! {
            ::pillar::ast::Projection::Aggregate(
                ::pillar::ast::AggregateFunction::Sum(#source.to_string())
            )
        },
        AggregateOp::Avg => quote! {
            ::pillar::ast::Projection::Aggregate(
                ::pillar::ast::AggregateFunction::Avg(#source.to_string())
            )
        },
        AggregateOp::Min => quote! {
            ::pillar::ast::Projection::Aggregate(
                ::pillar::ast::AggregateFunction::Min(#source.to_string())
            )
        },
        AggregateOp::Max => quote! {
            ::pillar::ast::Projection::Aggregate(
                ::pillar::ast::AggregateFunction::Max(#source.to_string())
            )
        },
    }
}

fn view_schema_impl(fields: &[FieldAttrs], attrs: &ViewAttrs) -> syn::Result<TokenStream> {
    let has_ddl = attrs.materialized
        || attrs.to.is_some()
        || attrs.engine.is_some()
        || attrs.partition_by.is_some()
        || attrs.options.as_ref().map(|o| !o.is_empty()).unwrap_or(false);

    if !has_ddl {
        return Ok(quote! {});
    }

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

    let create_stmt = if attrs.materialized || attrs.to.is_some() || attrs.engine.is_some() {
        let to_table_call = match &attrs.to {
            Some(t) => quote! { .to_table(#t) },
            None => quote! {},
        };

        quote! {
            ::pillar::ast::Statement::CreateMaterializedView(
                ::pillar::ast::CreateMaterializedViewStatement::new(
                    <Self as ::pillar::view::View>::view_name(),
                    <Self as ::pillar::view::ViewQuery>::query(),
                )
                .if_not_exists()
                #to_table_call
                #option_calls
            )
        }
    } else {
        quote! {
            ::pillar::ast::Statement::CreateView(
                ::pillar::ast::CreateViewStatement::new(
                    <Self as ::pillar::view::View>::view_name(),
                    <Self as ::pillar::view::ViewQuery>::query(),
                )
                .if_not_exists()
                #option_calls
            )
        }
    };

    Ok(quote! {
        impl ::pillar::view::ViewSchema for View {
            fn create_statement() -> ::pillar::ast::Statement {
                #create_stmt
            }
        }
    })
}
