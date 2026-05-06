use std::fmt::{Debug, Formatter, Result as FmtResult};
use async_trait::async_trait;
use futures::stream::BoxStream;
use arrow::record_batch::RecordBatch;

use crate::{errors::Error, ast::Statement, value::Value};


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
    List(Box<ColumnType>),
    Map(Box<ColumnType>, Box<ColumnType>),
    #[cfg(feature = "chrono")]
    Date,
    #[cfg(feature = "chrono")]
    Time,
    #[cfg(feature = "chrono")]
    DateTime,
    #[cfg(feature = "uuid")]
    Uuid,
}

#[derive(Debug, Clone)]
pub struct ColumnDef {
    pub name: &'static str,
    pub column_type: ColumnType,
    pub nullable: bool,
    pub primary_key: bool,
    pub unique: bool,
}

pub trait Model: Sized + Send + Sync {
    fn table_name() -> &'static str;
    fn columns() -> &'static [ColumnDef];
    fn from_record_batch(batch: RecordBatch) -> Result<Vec<Self>, Error>;
    fn to_record_batch(rows: &[Self]) -> Result<RecordBatch, Error>;

    fn primary_keys() -> Vec<&'static str> {
        Self::columns()
            .iter()
            .filter(|col| col.primary_key)
            .map(|col| col.name)
            .collect()
    }

    fn get_column(name: &str) -> Option<&'static ColumnDef> {
        Self::columns()
            .iter()
            .find(|col| col.name == name)
    }
}

pub trait MaterializedView: Sized + Send + Sync {
    fn view_name() -> &'static str;
    fn columns() -> &'static [ColumnDef];
    fn from_record_batch(batch: RecordBatch) -> Result<Vec<Self>, Error>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Feature {
    WindowFunctions,
    CommonTableExpressions,
    MaterializedViews,
    Partitioning,
    ArrayFunctions,
    JsonFunctions,
    ApproximateAggregates,
    NestedTypes,
}

#[derive(Debug, Clone)]
pub struct PreparedStatement {
    pub sql: String,
    pub params: Vec<Value>,
}

pub trait Dialect: Send + Sync {
    fn name(&self) -> &'static str;
    fn transpile(&self, statement: &Statement) -> Result<PreparedStatement, Error>;
    fn supports_feature(&self, feature: Feature) -> bool;
    fn quote_identifier(&self, identifier: &str) -> String;
    fn parameter_placeholder(&self, index: usize) -> String;
}

impl Debug for dyn Dialect {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "Dialect({})", self.name())
    }
}

#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub rows_affected: usize,
    pub metadata: Option<serde_json::Value>,
}

#[async_trait]
pub trait Database: Send + Sync {
    fn dialect(&self) -> &dyn Dialect;

    async fn execute(&self, statement: &Statement) -> Result<ExecutionResult, Error>;
    async fn query(&self, statement: &Statement) -> Result<RecordBatch, Error>;
    async fn query_stream(&self, statement: &Statement) -> Result<BoxStream<'_, Result<RecordBatch, Error>>, Error>;
}
