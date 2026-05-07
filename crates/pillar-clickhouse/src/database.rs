use std::collections::VecDeque;
use arrow::buffer::Buffer;
use arrow::ipc::reader::StreamDecoder;
use arrow::record_batch::RecordBatch;
use async_trait::async_trait;
use futures::stream::{self, Stream, BoxStream, StreamExt};

use pillar_core::{
    ast::Statement,
    database::Database,
    dialect::Dialect,
    errors::Error,
    types::{ExecutionResult, QueryResult},
    value::Value,
};

use crate::dialect::ClickHouseDialect;


/// A [`pillar_core::database::Database`](pillar_core::database::Database) implementation backed by ClickHouse.
pub struct ClickHouseDatabase {
    client: clickhouse::Client,
    dialect: ClickHouseDialect,
}

impl ClickHouseDatabase {
    pub fn new(client: clickhouse::Client) -> Self {
        Self { client, dialect: ClickHouseDialect }
    }

    pub fn builder(url: impl Into<String>) -> ClickHouseDatabaseBuilder {
        ClickHouseDatabaseBuilder::new(url)
    }

    fn bind_params(mut query: clickhouse::query::Query, params: &[Value]) -> clickhouse::query::Query {
        for param in params {
            query = match param {
                Value::Null => query.bind(Option::<String>::None),
                Value::Boolean(v) => query.bind(v),
                Value::Int8(v) => query.bind(v),
                Value::Int16(v) => query.bind(v),
                Value::Int32(v) => query.bind(v),
                Value::Int64(v) => query.bind(v),
                Value::UInt8(v) => query.bind(v),
                Value::UInt16(v) => query.bind(v),
                Value::UInt32(v) => query.bind(v),
                Value::UInt64(v) => query.bind(v),
                Value::Float32(v) => query.bind(v),
                Value::Float64(v) => query.bind(v),
                Value::String(v) => query.bind(v.as_str()),
                Value::Bytes(v) => query.bind(v.as_slice()),
                Value::List(_) | Value::Map(_) => query.bind(param.to_string()),
                #[cfg(feature = "chrono")]
                Value::Date(v) => query.bind(v),
                #[cfg(feature = "chrono")]
                Value::Time(v) => query.bind(v.to_string()),
                #[cfg(feature = "chrono")]
                Value::DateTime(v) => query.bind(v.format("%Y-%m-%d %H:%M:%S").to_string()),
                #[cfg(feature = "uuid")]
                Value::Uuid(v) => query.bind(v.to_string()),
            };
        }

        query
    }

    fn decode_stream(cursor: clickhouse::query::BytesCursor) -> impl Stream<Item = Result<RecordBatch, Error>> {
        stream::unfold(
            (cursor, StreamDecoder::new(), VecDeque::<RecordBatch>::new(), false),
            |(mut cursor, mut decoder, mut pending, done)| async move {
                if done {
                    return None;
                }

                if let Some(batch) = pending.pop_front() {
                    return Some((Ok(batch), (cursor, decoder, pending, false)));
                }

                loop {
                    match cursor.next().await {
                        Err(e) => return Some((Err(Error::connection(e.to_string())), (cursor, decoder, pending, true))),
                        Ok(None) => {
                            return match decoder.finish() {
                                Err(e) => Some((Err(Error::unexpected(e.to_string())), (cursor, decoder, pending, true))),
                                Ok(()) => None,
                            };
                        }
                        Ok(Some(chunk)) => {
                            let mut buf = Buffer::from(chunk.as_ref());

                            while !buf.is_empty() {
                                match decoder.decode(&mut buf) {
                                    Err(e) => return Some((Err(Error::unexpected(e.to_string())), (cursor, decoder, pending, true))),
                                    Ok(Some(batch)) => pending.push_back(batch),
                                    Ok(None) => {}
                                }
                            }

                            if let Some(batch) = pending.pop_front() {
                                return Some((Ok(batch), (cursor, decoder, pending, false)));
                            }
                        }
                    }
                }
            },
        )
    }
}

#[async_trait]
impl Database for ClickHouseDatabase {
    fn dialect(&self) -> &dyn Dialect {
        &self.dialect
    }

    async fn execute(&self, statement: &Statement) -> Result<ExecutionResult, Error> {
        let prepared = self.dialect.transpile(statement)?;

        Self::bind_params(self.client.query(&prepared.sql), &prepared.params)
            .execute()
            .await
            .map(|_| ExecutionResult { rows_affected: 0, metadata: None })
            .map_err(|e| Error::connection(e.to_string()))
    }

    async fn query(&self, statement: &Statement) -> Result<QueryResult, Error> {
        let prepared = self.dialect.transpile(statement)?;

        let cursor = Self::bind_params(self.client.query(&prepared.sql), &prepared.params)
            .fetch_bytes("ArrowStream")
            .map_err(|e| Error::connection(e.to_string()))?;

        let batches = Self::decode_stream(cursor)
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;

        match batches.len() {
            0 => Err(Error::unexpected("query returned no record batches")),
            1 => Ok(QueryResult::from(batches.into_iter().next().unwrap())),
            _ => arrow::compute::concat_batches(&batches[0].schema(), &batches)
                .map(QueryResult::from)
                .map_err(|e| Error::unexpected(e.to_string())),
        }
    }

    async fn query_stream(
        &self,
        statement: &Statement,
    ) -> Result<BoxStream<'_, Result<QueryResult, Error>>, Error> {
        let prepared = self.dialect.transpile(statement)?;

        Ok(Box::pin(
            Self::decode_stream(
                Self::bind_params(self.client.query(&prepared.sql), &prepared.params)
                    .fetch_bytes("ArrowStream")
                    .map_err(|e| Error::connection(e.to_string()))?,
            )
            .map(|r| r.map(QueryResult::from)),
        ))
    }
}

/// A builder for [`ClickHouseDatabase`](crate::database::ClickHouseDatabase).
pub struct ClickHouseDatabaseBuilder {
    url: String,
    database: Option<String>,
    username: Option<String>,
    password: Option<String>,
}

impl ClickHouseDatabaseBuilder {
    /// Creates a new builder targeting the given server URL.
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            database: None,
            username: None,
            password: None,
        }
    }

    /// Sets the database name.
    pub fn database(mut self, database: impl Into<String>) -> Self {
        self.database = Some(database.into());
        self
    }

    /// Sets the username.
    pub fn username(mut self, username: impl Into<String>) -> Self {
        self.username = Some(username.into());
        self
    }

    /// Sets the password.
    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    /// Builds the [`ClickHouseDatabase`](crate::database::ClickHouseDatabase).
    pub fn build(self) -> ClickHouseDatabase {
        let mut client = clickhouse::Client::default()
            .with_url(self.url);

        if let Some(db) = self.database {
            client = client.with_database(db);
        }

        if let Some(username) = self.username {
            client = client.with_user(username);
        }

        if let Some(password) = self.password {
            client = client.with_password(password);
        }

        ClickHouseDatabase::new(client)
    }
}
