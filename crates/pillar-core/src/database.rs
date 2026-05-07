use async_trait::async_trait;
use arrow::record_batch::RecordBatch;
use futures::stream::BoxStream;

use crate::{ast::Statement, dialect::Dialect, errors::Error};


/// The result of a statement that does not return rows.
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// Number of rows affected by the statement.
    pub rows_affected: usize,
    pub metadata: Option<serde_json::Value>,
}

/// A connection to a database backend.
#[async_trait]
pub trait Database: Send + Sync {
    /// Returns the [`Dialect`](crate::dialect::Dialect) for this connection.
    fn dialect(&self) -> &dyn Dialect;

    /// Executes a statement that does not return rows.
    async fn execute(&self, statement: &Statement) -> Result<ExecutionResult, Error>;

    /// Executes a query and returns all results as a single [`RecordBatch`].
    async fn query(&self, statement: &Statement) -> Result<RecordBatch, Error>;

    /// Executes a query and returns results as a stream of [`RecordBatch`] values.
    async fn query_stream(&self, statement: &Statement) -> Result<BoxStream<'_, Result<RecordBatch, Error>>, Error>;
}

/// Converts a concrete database type into a `&dyn` [`Database`](crate::database::Database) reference.
pub trait AsDynDatabase {
    fn as_dyn(&self) -> &dyn Database;
}

impl<D: Database> AsDynDatabase for D {
    fn as_dyn(&self) -> &dyn Database {
        self
    }
}
