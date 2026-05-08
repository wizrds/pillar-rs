use std::sync::Arc;
use async_trait::async_trait;
use futures::stream::BoxStream;

use crate::{
    ast::Statement,
    dialect::Dialect,
    errors::Error,
    types::{ExecutionResult, QueryResult},
};


/// A connection to a database backend.
#[async_trait]
pub trait Database: Send + Sync {
    /// Returns the [`Dialect`](crate::dialect::Dialect) for this connection.
    fn dialect(&self) -> &dyn Dialect;

    /// Executes a statement that does not return rows.
    async fn execute(&self, statement: &Statement) -> Result<ExecutionResult, Error>;

    /// Executes a query and returns all results as a single [`QueryResult`].
    async fn query(&self, statement: &Statement) -> Result<QueryResult, Error>;

    /// Executes a query and returns results as a stream of [`QueryResult`] values.
    async fn query_stream(&self, statement: &Statement) -> Result<BoxStream<'_, Result<QueryResult, Error>>, Error>;
}

#[async_trait]
impl Database for &dyn Database {
    fn dialect(&self) -> &dyn Dialect {
        (*self).dialect()
    }

    async fn execute(&self, statement: &Statement) -> Result<ExecutionResult, Error> {
        (*self).execute(statement).await
    }

    async fn query(&self, statement: &Statement) -> Result<QueryResult, Error> {
        (*self).query(statement).await
    }

    async fn query_stream(&self, statement: &Statement) -> Result<BoxStream<'_, Result<QueryResult, Error>>, Error> {
        (*self).query_stream(statement).await
    }
}

#[async_trait]
impl Database for Arc<dyn Database> {
    fn dialect(&self) -> &dyn Dialect {
        (**self).dialect()
    }

    async fn execute(&self, statement: &Statement) -> Result<ExecutionResult, Error> {
        (**self).execute(statement).await
    }

    async fn query(&self, statement: &Statement) -> Result<QueryResult, Error> {
        (**self).query(statement).await
    }

    async fn query_stream(&self, statement: &Statement) -> Result<BoxStream<'_, Result<QueryResult, Error>>, Error> {
        (**self).query_stream(statement).await
    }
}

#[async_trait]
impl Database for Box<dyn Database> {
    fn dialect(&self) -> &dyn Dialect {
        (**self).dialect()
    }

    async fn execute(&self, statement: &Statement) -> Result<ExecutionResult, Error> {
        (**self).execute(statement).await
    }

    async fn query(&self, statement: &Statement) -> Result<QueryResult, Error> {
        (**self).query(statement).await
    }

    async fn query_stream(&self, statement: &Statement) -> Result<BoxStream<'_, Result<QueryResult, Error>>, Error> {
        (**self).query_stream(statement).await
    }
}

#[async_trait]
impl<D: Database> Database for Arc<D> {
    fn dialect(&self) -> &dyn Dialect {
        (**self).dialect()
    }

    async fn execute(&self, statement: &Statement) -> Result<ExecutionResult, Error> {
        (**self).execute(statement).await
    }

    async fn query(&self, statement: &Statement) -> Result<QueryResult, Error> {
        (**self).query(statement).await
    }

    async fn query_stream(&self, statement: &Statement) -> Result<BoxStream<'_, Result<QueryResult, Error>>, Error> {
        (**self).query_stream(statement).await
    }
}

#[async_trait]
impl<D: Database> Database for Box<D> {
    fn dialect(&self) -> &dyn Dialect {
        (**self).dialect()
    }

    async fn execute(&self, statement: &Statement) -> Result<ExecutionResult, Error> {
        (**self).execute(statement).await
    }

    async fn query(&self, statement: &Statement) -> Result<QueryResult, Error> {
        (**self).query(statement).await
    }

    async fn query_stream(&self, statement: &Statement) -> Result<BoxStream<'_, Result<QueryResult, Error>>, Error> {
        (**self).query_stream(statement).await
    }
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

