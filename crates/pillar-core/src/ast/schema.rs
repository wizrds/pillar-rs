use std::collections::HashMap;

use crate::{
    value::Value,
    ast::{
        select::SelectStatement,
        ttl::TtlClause,
    }
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
}

/// The aggregate function and argument types stored in an [`AggregateState`](crate::ast::ColumnType::AggregateState) column.
#[derive(Debug, Clone, PartialEq)]
pub struct AggregateStateFunction {
    pub function: AggregateFn,
    pub arg_types: Vec<ColumnType>,
}

impl AggregateStateFunction {
    /// Creates a new [`AggregateStateFunction`](crate::ast::AggregateStateFunction) with the given function and argument types.
    pub fn new(function: AggregateFn, arg_types: Vec<ColumnType>) -> Self {
        Self { function, arg_types }
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

/// Defines a single column in a [`CreateTableStatement`](crate::ast::CreateTableStatement) or [`AlterTableStatement`](crate::ast::AlterTableStatement).
#[derive(Debug, Clone, PartialEq)]
pub struct ColumnDefinition {
    pub name: String,
    pub data_type: ColumnType,
    pub nullable: bool,
    pub primary_key: bool,
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
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            columns: Vec::new(),
            if_not_exists: false,
            options: HashMap::new(),
            ttl: None,
        }
    }

    /// Sets the column definitions for the table.
    pub fn columns(mut self, columns: Vec<ColumnDefinition>) -> Self {
        self.columns = columns;
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
    pub drop_columns: Vec<String>,
    pub ttl: Option<TtlClause>,
}

impl AlterTableStatement {
    /// Creates a new [`AlterTableStatement`](crate::ast::AlterTableStatement) for the given table name.
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), add_columns: Vec::new(), drop_columns: Vec::new(), ttl: None }
    }

    /// Sets the columns to add.
    pub fn add_columns(mut self, columns: Vec<ColumnDefinition>) -> Self {
        self.add_columns = columns;
        self
    }

    /// Sets the column names to drop.
    pub fn drop_columns(mut self, columns: Vec<String>) -> Self {
        self.drop_columns = columns;
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
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), if_exists: false }
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
    pub fn new(name: impl Into<String>, query: SelectStatement) -> Self {
        Self {
            name: name.into(),
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
    pub fn new(name: impl Into<String>, query: SelectStatement) -> Self {
        Self {
            name: name.into(),
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
    pub fn to_table(mut self, table: impl Into<String>) -> Self {
        self.to_table = Some(table.into());
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
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), if_exists: false, materialized: false }
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
