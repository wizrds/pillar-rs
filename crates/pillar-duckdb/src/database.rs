use std::sync::{Arc, Mutex};
use async_trait::async_trait;
use futures::stream::{self, BoxStream};
use arrow::{compute::concat_batches, record_batch::RecordBatch};
use pillar_core::{
    ast::Statement,
    errors::Error,
    database::{Database, ExecutionResult},
    dialect::Dialect,
};

use crate::{dialect::DuckDbDialect, normalize::BatchNormalizer, value::DuckDbValue};


/// A [`pillar_core::database::Database`](pillar_core::database::Database) implementation backed by DuckDB.
pub struct DuckDbDatabase {
    conn: Arc<Mutex<duckdb::Connection>>,
    dialect: DuckDbDialect,
    normalizer: BatchNormalizer,
}

impl DuckDbDatabase {
    /// Creates a new [`DuckDbDatabase`](crate::database::DuckDbDatabase) from an existing DuckDB connection.
    pub fn from_connection(conn: duckdb::Connection) -> Self {
        Self {
            conn: Arc::new(Mutex::new(conn)),
            dialect: DuckDbDialect,
            normalizer: BatchNormalizer::new(),
        }
    }

    /// Opens an in-memory DuckDB database.
    pub async fn in_memory() -> Result<Self, Error> {
        blocking::unblock(|| {
            duckdb::Connection::open_in_memory()
                .map(DuckDbDatabase::from_connection)
                .map_err(|e| Error::connection(e.to_string()))
        })
        .await
    }

    /// Opens a DuckDB database at the given file path.
    pub async fn open(path: impl AsRef<std::path::Path> + Send + 'static) -> Result<Self, Error> {
        blocking::unblock(|| {
            duckdb::Connection::open(path)
                .map(DuckDbDatabase::from_connection)
                .map_err(|e| Error::connection(e.to_string()))
        })
        .await
    }
}

#[async_trait]
impl Database for DuckDbDatabase {
    fn dialect(&self) -> &dyn Dialect {
        &self.dialect
    }

    async fn execute(&self, statement: &Statement) -> Result<ExecutionResult, Error> {
        let prepared = self.dialect.transpile(statement)?;
        let conn = Arc::clone(&self.conn);

        blocking::unblock(move || {
            let guard = conn.lock().map_err(|e| Error::connection(e.to_string()))?;

            if prepared.params.is_empty() {
                guard
                    .execute_batch(&prepared.sql)
                    .map(|_| ExecutionResult { rows_affected: 0, metadata: None })
                    .map_err(|e| Error::connection(e.to_string()))
            } else {
                guard
                    .execute(
                        &prepared.sql,
                        duckdb::params_from_iter(prepared.params.iter().map(DuckDbValue::from)),
                    )
                    .map(|rows_affected| ExecutionResult { rows_affected, metadata: None })
                    .map_err(|e| Error::connection(e.to_string()))
            }
        })
        .await
    }

    async fn query(&self, statement: &Statement) -> Result<RecordBatch, Error> {
        let prepared = self.dialect.transpile(statement)?;
        let conn = Arc::clone(&self.conn);

        blocking::unblock(move || {
            let guard = conn
                .lock()
                .map_err(|e| Error::connection(e.to_string()))?;

            let mut stmt = guard
                .prepare(&prepared.sql)
                .map_err(|e| Error::connection(e.to_string()))?;

            let arrow = stmt
                .query_arrow(duckdb::params_from_iter(
                    prepared.params
                        .iter()
                        .map(DuckDbValue::from),
                ))
                .map_err(|e| Error::connection(e.to_string()))?;

            concat_batches(&arrow.get_schema(), &arrow.collect::<Vec<_>>())
                .map_err(|e| Error::unexpected(e.to_string()))
        })
        .await
        .and_then(|batch| self.normalizer.normalize(batch))
    }

    async fn query_stream(
        &self,
        statement: &Statement,
    ) -> Result<BoxStream<'_, Result<RecordBatch, Error>>, Error> {
        let prepared = self.dialect.transpile(statement)?;
        let conn = Arc::clone(&self.conn);

        Ok(Box::pin(stream::iter(
            blocking::unblock(move || {
                let guard = conn
                    .lock()
                    .map_err(|e| Error::connection(e.to_string()))?;

                let mut stmt = guard
                    .prepare(&prepared.sql)
                    .map_err(|e| Error::connection(e.to_string()))?;

                stmt.query_arrow(duckdb::params_from_iter(
                    prepared.params
                        .iter()
                        .map(DuckDbValue::from),
                ))
                .map(|arrow| arrow.collect::<Vec<_>>())
                .map_err(|e| Error::connection(e.to_string()))
            })
            .await?
            .into_iter()
            .map(|batch| self.normalizer.normalize(batch))
        )))
    }
}
