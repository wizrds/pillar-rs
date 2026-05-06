use async_trait::async_trait;
use arrow::record_batch::RecordBatch;
use futures::stream::BoxStream;

use crate::{ast::Statement, dialect::Dialect, errors::Error};


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

pub trait AsDynDatabase {
    fn as_dyn(&self) -> &dyn Database;
}

impl<D: Database> AsDynDatabase for D {
    fn as_dyn(&self) -> &dyn Database {
        self
    }
}
