use arrow::buffer::Buffer;
use arrow::ipc::reader::StreamDecoder;
use arrow::record_batch::RecordBatch;
use async_trait::async_trait;
use futures::stream::{self, Stream, BoxStream, StreamExt};
use futures::AsyncReadExt;

use pillar_core::{
    ast::Statement,
    database::{Database, ExecutionResult},
    dialect::Dialect,
    errors::Error,
    value::Value,
};

use crate::dialect::ClickHouseDialect;


const CHUNK_SIZE: usize = 65536;

pub struct ClickHouseDatabase {
    client: clickhouse::Client,
    dialect: ClickHouseDialect,
}

impl ClickHouseDatabase {
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
                Value::DateTime(v) => query.bind(v),
                #[cfg(feature = "uuid")]
                Value::Uuid(v) => query.bind(v.to_string()),
            };
        }
        query
    }

    fn decode_stream(cursor: clickhouse::query::BytesCursor) -> impl Stream<Item = Result<RecordBatch, Error>> {
        stream::unfold((cursor, StreamDecoder::new(), false), |(mut cursor, mut decoder, done)| async move {
            if done {
                return None;
            }

            let mut chunk = vec![0u8; CHUNK_SIZE];

            loop {
                match cursor.read(&mut chunk).await {
                    Err(e) => return Some((Err(Error::connection(e.to_string())), (cursor, decoder, true))),
                    Ok(0) => {
                        if let Err(e) = decoder.finish() {
                            return Some((Err(Error::unexpected(e.to_string())), (cursor, decoder, true)));
                        }

                        return None;
                    }
                    Ok(n) => {
                        let mut buf = Buffer::from(&chunk[..n]);

                        while !buf.is_empty() {
                            match decoder.decode(&mut buf) {
                                Err(e) => return Some((Err(Error::unexpected(e.to_string())), (cursor, decoder, true))),
                                Ok(Some(batch)) => return Some((Ok(batch), (cursor, decoder, false))),
                                Ok(None) => {}
                            }
                        }
                    }
                }
            }
        })
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

    async fn query(&self, statement: &Statement) -> Result<RecordBatch, Error> {
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
            1 => Ok(batches.into_iter().next().unwrap()),
            _ => arrow::compute::concat_batches(&batches[0].schema(), &batches)
                .map_err(|e| Error::unexpected(e.to_string())),
        }
    }

    async fn query_stream(
        &self,
        statement: &Statement,
    ) -> Result<BoxStream<'_, Result<RecordBatch, Error>>, Error> {
        let prepared = self.dialect.transpile(statement)?;

        Ok(Box::pin(Self::decode_stream(
            Self::bind_params(self.client.query(&prepared.sql), &prepared.params)
                .fetch_bytes("ArrowStream")
                .map_err(|e| Error::connection(e.to_string()))?
        )))
    }
}

pub struct ClickHouseDatabaseBuilder {
    url: String,
    database: Option<String>,
    user: Option<String>,
    password: Option<String>,
}

impl ClickHouseDatabaseBuilder {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            database: None,
            user: None,
            password: None,
        }
    }

    pub fn database(mut self, database: impl Into<String>) -> Self {
        self.database = Some(database.into());
        self
    }

    pub fn user(mut self, user: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self
    }

    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    pub fn build(self) -> ClickHouseDatabase {
        let mut client = clickhouse::Client::default()
            .with_url(self.url);

        if let Some(db) = self.database {
            client = client.with_database(db);
        }

        if let Some(user) = self.user {
            client = client.with_user(user);
        }

        if let Some(password) = self.password {
            client = client.with_password(password);
        }

        ClickHouseDatabase { client, dialect: ClickHouseDialect }
    }
}
