use std::collections::HashMap;
use std::sync::Arc;

use arrow::datatypes::{DataType, Field};

#[cfg(feature = "chrono")]
use arrow::datatypes::TimeUnit;

use crate::{
    errors::Error,
    value::Value,
    ast::{
        refs::{ColumnRef, TableRef},
        select::SelectStatement,
        ttl::TtlClause,
    },
};


/// The data type of a column in the database schema.
#[derive(Debug, Clone, PartialEq)]
pub enum ColumnType {
    Boolean,
    Int8,
    Int16,
    Int32,
    Int64,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Float32,
    Float64,
    String,
    Binary,
    /// A list of values of the given type.
    List(Box<ColumnType>),
    /// A map from keys of one type to values of another.
    Map(Box<ColumnType>, Box<ColumnType>),
    #[cfg(feature = "chrono")]
    Date,
    #[cfg(feature = "chrono")]
    Time,
    #[cfg(feature = "chrono")]
    DateTime,
    #[cfg(feature = "uuid")]
    Uuid,
    /// High-precision timestamp with sub-second precision digits (0 to 9).
    DateTime64 { precision: u8 },
    /// String optimized for low-cardinality data such as enums or status codes.
    LowCardinalityString,
    /// Fixed-length string of exactly `n` bytes.
    FixedString(u32),
    /// Stores intermediate aggregate state for incremental rollup tables.
    AggregateState(AggregateStateFunction),
    /// Explicit nullable wrapper for backends that require it in the type position.
    Nullable(Box<ColumnType>),
    /// A raw type string passed through to the backend as-is.
    Custom(String),
}

impl ColumnType {
    /// A list of values of the given type.
    pub fn list(inner: impl Into<ColumnType>) -> Self {
        Self::List(Box::new(inner.into()))
    }

    /// A map from keys of one type to values of another.
    pub fn map(key: impl Into<ColumnType>, value: impl Into<ColumnType>) -> Self {
        Self::Map(Box::new(key.into()), Box::new(value.into()))
    }

    /// Wraps this type in an explicit nullable marker.
    pub fn nullable(inner: impl Into<ColumnType>) -> Self {
        Self::Nullable(Box::new(inner.into()))
    }

    /// An aggregate state column for the given function and argument types.
    pub fn aggregate_state(
        function: AggregateFn,
        arg_types: impl IntoIterator<Item = ColumnType>,
    ) -> Self {
        Self::AggregateState(AggregateStateFunction {
            function,
            arg_types: arg_types.into_iter().collect(),
        })
    }

    /// A fixed-length string of exactly `n` bytes.
    pub fn fixed_string(n: u32) -> Self {
        Self::FixedString(n)
    }

    /// A high-precision timestamp with the given sub-second precision digits (0–9).
    pub fn datetime64(precision: u8) -> Self {
        Self::DateTime64 { precision }
    }

    /// A raw type string passed through to the backend as-is.
    pub fn custom(s: impl Into<String>) -> Self {
        Self::Custom(s.into())
    }

    /// Maps this column type to its Arrow [`DataType`](arrow::datatypes::DataType) equivalent.
    ///
    /// Returns an error for types that have no valid Arrow representation.
    pub fn to_arrow_data_type(&self) -> Result<DataType, Error> {
        match self {
            ColumnType::Boolean => Ok(DataType::Boolean),
            ColumnType::Int8 => Ok(DataType::Int8),
            ColumnType::Int16 => Ok(DataType::Int16),
            ColumnType::Int32 => Ok(DataType::Int32),
            ColumnType::Int64 => Ok(DataType::Int64),
            ColumnType::UInt8 => Ok(DataType::UInt8),
            ColumnType::UInt16 => Ok(DataType::UInt16),
            ColumnType::UInt32 => Ok(DataType::UInt32),
            ColumnType::UInt64 => Ok(DataType::UInt64),
            ColumnType::Float32 => Ok(DataType::Float32),
            ColumnType::Float64 => Ok(DataType::Float64),
            ColumnType::String | ColumnType::LowCardinalityString => Ok(DataType::LargeUtf8),
            ColumnType::Binary => Ok(DataType::LargeBinary),
            ColumnType::FixedString(n) => Ok(DataType::FixedSizeBinary(*n as i32)),
            ColumnType::List(inner) => Ok(DataType::LargeList(Arc::new(
                Field::new("item", inner.to_arrow_data_type()?, true),
            ))),
            ColumnType::Map(key, value) => Ok(DataType::Map(
                Arc::new(Field::new(
                    "entries",
                    DataType::Struct(
                        vec![
                            Field::new("key", key.to_arrow_data_type()?, false),
                            Field::new("value", value.to_arrow_data_type()?, true),
                        ]
                        .into(),
                    ),
                    false,
                )),
                false,
            )),
            #[cfg(feature = "chrono")]
            ColumnType::Date => Ok(DataType::Date32),
            #[cfg(feature = "chrono")]
            ColumnType::Time => Ok(DataType::Time64(TimeUnit::Nanosecond)),
            #[cfg(feature = "chrono")]
            ColumnType::DateTime | ColumnType::DateTime64 { .. } => {
                Ok(DataType::Timestamp(TimeUnit::Nanosecond, Some("UTC".into())))
            }
            #[cfg(feature = "uuid")]
            ColumnType::Uuid => Ok(DataType::FixedSizeBinary(16)),
            ColumnType::Nullable(inner) => inner.to_arrow_data_type(),
            _ => Err(Error::serialization(
                "column type has no Arrow equivalent",
            )),
        }
    }
}

/// The aggregate function and argument types stored in an [`AggregateState`](crate::ast::ColumnType::AggregateState) column.
#[derive(Debug, Clone, PartialEq)]
pub struct AggregateStateFunction {
    pub function: AggregateFn,
    pub arg_types: Vec<ColumnType>,
}

impl AggregateStateFunction {
    /// Creates a new [`AggregateStateFunction`](crate::ast::AggregateStateFunction) with the given function and argument types.
    pub fn new(function: AggregateFn, arg_types: impl IntoIterator<Item = ColumnType>) -> Self {
        Self { function, arg_types: arg_types.into_iter().collect() }
    }
}

/// The aggregate function variant for an [`AggregateStateFunction`](crate::ast::AggregateStateFunction).
#[derive(Debug, Clone, PartialEq)]
pub enum AggregateFn {
    Count,
    Sum,
    Avg,
    Min,
    Max,
    Uniq,
    Quantile(f64),
    TopK(u32),
    Histogram(u32),
    Custom(String),
}

impl AggregateFn {
    /// `COUNT` aggregate.
    pub fn count() -> Self {
        Self::Count
    }

    /// `SUM` aggregate.
    pub fn sum() -> Self {
        Self::Sum
    }

    /// `AVG` aggregate.
    pub fn avg() -> Self {
        Self::Avg
    }

    /// `MIN` aggregate.
    pub fn min() -> Self {
        Self::Min
    }

    /// `MAX` aggregate.
    pub fn max() -> Self {
        Self::Max
    }

    /// `uniq` aggregate or equivalent.
    pub fn uniq() -> Self {
        Self::Uniq
    }

    /// `QUANTILE(level)` aggregate or equivalent.
    pub fn quantile(level: f64) -> Self {
        Self::Quantile(level)
    }

    /// `topK(k)` aggregate or equivalent.
    pub fn top_k(k: u32) -> Self {
        Self::TopK(k)
    }

    /// `histogram(bins)` aggregate or equivalent.
    pub fn histogram(bins: u32) -> Self {
        Self::Histogram(bins)
    }

    /// A raw aggregate function name passed through to the backend as-is.
    pub fn custom(s: impl Into<String>) -> Self {
        Self::Custom(s.into())
    }
}

/// Defines a single column in a [`CreateTableStatement`](crate::ast::CreateTableStatement) or [`AlterTableStatement`](crate::ast::AlterTableStatement).
#[derive(Debug, Clone, PartialEq)]
pub struct ColumnDefinition {
    pub name: String,
    pub data_type: ColumnType,
    pub nullable: bool,
    pub primary_key: bool,
    pub unique: bool,
    pub default: Option<Value>,
}

impl ColumnDefinition {
    /// Creates a non-nullable, non-primary-key column with no default.
    pub fn new(name: impl Into<String>, data_type: ColumnType) -> Self {
        Self {
            name: name.into(),
            data_type,
            nullable: false,
            primary_key: false,
            unique: false,
            default: None,
        }
    }

    /// Marks the column as nullable.
    pub fn nullable(mut self) -> Self {
        self.nullable = true;
        self
    }

    /// Marks the column as a primary key.
    pub fn primary_key(mut self) -> Self {
        self.primary_key = true;
        self
    }

    /// Marks the column as unique.
    pub fn unique(mut self) -> Self {
        self.unique = true;
        self
    }

    /// Sets a default value for the column.
    pub fn default(mut self, value: impl Into<Value>) -> Self {
        self.default = Some(value.into());
        self
    }
}

/// AST node for a `CREATE TABLE` statement.
#[derive(Debug, Clone, PartialEq)]
pub struct CreateTableStatement {
    pub name: String,
    pub columns: Vec<ColumnDefinition>,
    pub if_not_exists: bool,
    pub options: HashMap<String, String>,
    pub ttl: Option<TtlClause>,
}

impl CreateTableStatement {
    /// Creates a new [`CreateTableStatement`](crate::ast::CreateTableStatement) for the given table name.
    pub fn new(name: impl Into<TableRef>) -> Self {
        Self {
            name: name.into().name,
            columns: Vec::new(),
            if_not_exists: false,
            options: HashMap::new(),
            ttl: None,
        }
    }

    /// Sets the column definitions for the table.
    pub fn columns(mut self, columns: impl IntoIterator<Item = ColumnDefinition>) -> Self {
        self.columns = columns.into_iter().collect();
        self
    }

    /// Adds a single column definition.
    pub fn column(mut self, column: ColumnDefinition) -> Self {
        self.columns.push(column);
        self
    }

    /// Adds `IF NOT EXISTS` to the statement.
    pub fn if_not_exists(mut self) -> Self {
        self.if_not_exists = true;
        self
    }

    /// Adds a backend-specific table option.
    pub fn option(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.options.insert(key.into(), value.into());
        self
    }

    /// Sets the TTL rule for the table.
    pub fn ttl(mut self, ttl: TtlClause) -> Self {
        self.ttl = Some(ttl);
        self
    }
}

/// AST node for an `ALTER TABLE` statement.
#[derive(Debug, Clone, PartialEq)]
pub struct AlterTableStatement {
    pub name: String,
    pub add_columns: Vec<ColumnDefinition>,
    pub drop_columns: Vec<ColumnRef>,
    pub ttl: Option<TtlClause>,
}

impl AlterTableStatement {
    /// Creates a new [`AlterTableStatement`](crate::ast::AlterTableStatement) for the given table name.
    pub fn new(name: impl Into<TableRef>) -> Self {
        Self { name: name.into().name, add_columns: Vec::new(), drop_columns: Vec::new(), ttl: None }
    }

    /// Sets the columns to add.
    pub fn add_columns(mut self, columns: impl IntoIterator<Item = ColumnDefinition>) -> Self {
        self.add_columns = columns.into_iter().collect();
        self
    }

    /// Adds a single column.
    pub fn add_column(mut self, column: ColumnDefinition) -> Self {
        self.add_columns.push(column);
        self
    }

    /// Sets the column names to drop.
    pub fn drop_columns(mut self, columns: impl IntoIterator<Item = impl Into<ColumnRef>>) -> Self {
        self.drop_columns = columns.into_iter().map(Into::into).collect();
        self
    }

    /// Drops a single column by name.
    pub fn drop_column(mut self, column: impl Into<ColumnRef>) -> Self {
        self.drop_columns.push(column.into());
        self
    }

    /// Sets the TTL rule for the table.
    pub fn ttl(mut self, ttl: TtlClause) -> Self {
        self.ttl = Some(ttl);
        self
    }
}

/// AST node for a `DROP TABLE` statement.
#[derive(Debug, Clone, PartialEq)]
pub struct DropTableStatement {
    pub name: String,
    pub if_exists: bool,
}

impl DropTableStatement {
    /// Creates a new [`DropTableStatement`](crate::ast::DropTableStatement) for the given table name.
    pub fn new(name: impl Into<TableRef>) -> Self {
        Self { name: name.into().name, if_exists: false }
    }

    /// Adds `IF EXISTS` to the statement.
    pub fn if_exists(mut self) -> Self {
        self.if_exists = true;
        self
    }
}

/// AST node for a `CREATE VIEW` statement.
#[derive(Debug, Clone, PartialEq)]
pub struct CreateViewStatement {
    pub name: String,
    pub query: SelectStatement,
    pub or_replace: bool,
    pub if_not_exists: bool,
    pub options: HashMap<String, String>,
}

impl CreateViewStatement {
    /// Creates a new [`CreateViewStatement`](crate::ast::CreateViewStatement) for the given view name and query.
    pub fn new(name: impl Into<TableRef>, query: SelectStatement) -> Self {
        Self {
            name: name.into().name,
            query,
            or_replace: false,
            if_not_exists: false,
            options: HashMap::new(),
        }
    }

    /// Adds `OR REPLACE` to the statement.
    pub fn or_replace(mut self) -> Self {
        self.or_replace = true;
        self
    }

    /// Adds `IF NOT EXISTS` to the statement.
    pub fn if_not_exists(mut self) -> Self {
        self.if_not_exists = true;
        self
    }

    /// Adds a backend-specific view option.
    pub fn option(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.options.insert(key.into(), value.into());
        self
    }
}

/// AST node for a `CREATE MATERIALIZED VIEW` statement.
#[derive(Debug, Clone, PartialEq)]
pub struct CreateMaterializedViewStatement {
    pub name: String,
    pub query: SelectStatement,
    pub or_replace: bool,
    pub if_not_exists: bool,
    pub to_table: Option<String>,
    pub options: HashMap<String, String>,
}

impl CreateMaterializedViewStatement {
    /// Creates a new [`CreateMaterializedViewStatement`](crate::ast::CreateMaterializedViewStatement) for the given view name and query.
    pub fn new(name: impl Into<TableRef>, query: SelectStatement) -> Self {
        Self {
            name: name.into().name,
            query,
            or_replace: false,
            if_not_exists: false,
            to_table: None,
            options: HashMap::new(),
        }
    }

    /// Adds `OR REPLACE` to the statement.
    pub fn or_replace(mut self) -> Self {
        self.or_replace = true;
        self
    }

    /// Adds `IF NOT EXISTS` to the statement.
    pub fn if_not_exists(mut self) -> Self {
        self.if_not_exists = true;
        self
    }

    /// Routes materialized view output to an existing table.
    pub fn to_table(mut self, table: impl Into<TableRef>) -> Self {
        self.to_table = Some(table.into().name);
        self
    }

    /// Adds a backend-specific view option.
    pub fn option(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.options.insert(key.into(), value.into());
        self
    }
}

/// AST node for a `DROP VIEW` statement.
#[derive(Debug, Clone, PartialEq)]
pub struct DropViewStatement {
    pub name: String,
    pub if_exists: bool,
    pub materialized: bool,
}

impl DropViewStatement {
    /// Creates a new [`DropViewStatement`](crate::ast::DropViewStatement) for the given view name.
    pub fn new(name: impl Into<TableRef>) -> Self {
        Self { name: name.into().name, if_exists: false, materialized: false }
    }

    /// Adds `IF EXISTS` to the statement.
    pub fn if_exists(mut self) -> Self {
        self.if_exists = true;
        self
    }

    /// Marks this as a `DROP MATERIALIZED VIEW` statement.
    pub fn materialized(mut self) -> Self {
        self.materialized = true;
        self
    }
}
