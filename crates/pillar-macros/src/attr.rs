use darling::{ast, util, FromDeriveInput, FromField, FromMeta};


/// Strongly typed TTL attribute: `#[pillar(ttl(column = "...", interval = N, unit = "day"))]`.
#[derive(Clone, FromMeta)]
pub struct TtlAttr {
    pub column: String,
    pub interval: u32,
    pub unit: TtlUnit,
}

/// Time unit for a TTL interval.
#[derive(Clone)]
pub enum TtlUnit {
    Second,
    Minute,
    Hour,
    Day,
    Week,
    Month,
    Year,
}

impl FromMeta for TtlUnit {
    fn from_string(value: &str) -> darling::Result<Self> {
        match value.to_lowercase().trim_end_matches('s') {
            "second" => Ok(Self::Second),
            "minute" => Ok(Self::Minute),
            "hour" => Ok(Self::Hour),
            "day" => Ok(Self::Day),
            "week" => Ok(Self::Week),
            "month" => Ok(Self::Month),
            "year" => Ok(Self::Year),
            _ => Err(darling::Error::unknown_value(value)),
        }
    }
}

/// Aggregate function applied to a field in a materialized view query.
#[derive(Clone)]
pub enum AggregateOp {
    Count,
    Sum,
    Avg,
    Min,
    Max,
}

impl FromMeta for AggregateOp {
    fn from_string(value: &str) -> darling::Result<Self> {
        match value.to_lowercase().as_str() {
            "count" => Ok(Self::Count),
            "sum" => Ok(Self::Sum),
            "avg" => Ok(Self::Avg),
            "min" => Ok(Self::Min),
            "max" => Ok(Self::Max),
            _ => Err(darling::Error::unknown_value(value)),
        }
    }
}

#[derive(FromDeriveInput)]
#[darling(attributes(pillar), supports(struct_named))]
pub struct ModelAttrs {
    pub ident: syn::Ident,
    pub data: ast::Data<util::Ignored, FieldAttrs>,
    pub table: Option<String>,
    pub engine: Option<String>,
    pub partition_by: Option<String>,
    pub ttl: Option<TtlAttr>,
    /// Catch-all for backend-specific options not covered by named fields.
    pub options: Option<std::collections::HashMap<String, String>>,
}

#[derive(FromDeriveInput)]
#[darling(attributes(pillar), supports(struct_named))]
pub struct ViewAttrs {
    pub ident: syn::Ident,
    pub data: ast::Data<util::Ignored, FieldAttrs>,
    pub view: Option<String>,
    pub from: Option<String>,
    pub filter: Option<String>,
    /// Routes MV output to this table (ClickHouse `TO table`; ignored by DuckDB).
    pub to: Option<String>,
    pub engine: Option<String>,
    pub partition_by: Option<String>,
    /// Catch-all for backend-specific options not covered by named fields.
    pub options: Option<std::collections::HashMap<String, String>>,
}

#[derive(Clone, FromField)]
#[darling(attributes(pillar))]
pub struct FieldAttrs {
    pub ident: Option<syn::Ident>,
    pub ty: syn::Type,
    pub column: Option<String>,
    pub column_type: Option<String>,

    #[darling(default)]
    pub primary_key: bool,

    #[darling(default)]
    pub unique: bool,

    #[darling(default)]
    pub skip: bool,

    /// Marks this field as part of the ORDER BY key.
    #[darling(default)]
    pub order_by: bool,

    /// Aggregate function applied to this field in a materialized view query.
    pub aggregate: Option<AggregateOp>,

    /// Source column name for aggregate projections. Defaults to the field name if not set.
    pub source: Option<String>,
}
