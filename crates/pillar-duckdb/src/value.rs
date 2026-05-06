#[cfg(feature = "chrono")]
use chrono::{NaiveDate, Timelike};

use pillar_core::value::Value;


pub(crate) struct DuckDbValue(pub(crate) duckdb::types::Value);

impl From<&Value> for DuckDbValue {
    fn from(value: &Value) -> Self {
        DuckDbValue(match value {
            Value::Null => duckdb::types::Value::Null,
            Value::Boolean(b) => duckdb::types::Value::Boolean(*b),
            Value::Int8(i) => duckdb::types::Value::TinyInt(*i),
            Value::Int16(i) => duckdb::types::Value::SmallInt(*i),
            Value::Int32(i) => duckdb::types::Value::Int(*i),
            Value::Int64(i) => duckdb::types::Value::BigInt(*i),
            Value::UInt8(u) => duckdb::types::Value::UTinyInt(*u),
            Value::UInt16(u) => duckdb::types::Value::USmallInt(*u),
            Value::UInt32(u) => duckdb::types::Value::UInt(*u),
            Value::UInt64(u) => duckdb::types::Value::UBigInt(*u),
            Value::Float32(f) => duckdb::types::Value::Float(*f),
            Value::Float64(f) => duckdb::types::Value::Double(*f),
            Value::String(s) => duckdb::types::Value::Text(s.clone()),
            Value::Bytes(b) => duckdb::types::Value::Blob(b.clone()),
            Value::List(items) => duckdb::types::Value::List(
                items.iter().map(|v| DuckDbValue::from(v).0).collect(),
            ),
            Value::Map(pairs) => duckdb::types::Value::List(
                pairs
                    .iter()
                    .flat_map(|(k, v)| [DuckDbValue::from(k).0, DuckDbValue::from(v).0])
                    .collect(),
            ),

            #[cfg(feature = "chrono")]
            Value::Date(d) => duckdb::types::Value::Date32(
                d.signed_duration_since(NaiveDate::from_ymd_opt(1970, 1, 1).unwrap())
                    .num_days() as i32,
            ),

            #[cfg(feature = "chrono")]
            Value::Time(t) => duckdb::types::Value::Time64(
                duckdb::types::TimeUnit::Nanosecond,
                t.num_seconds_from_midnight() as i64 * 1_000_000_000 + t.nanosecond() as i64,
            ),

            #[cfg(feature = "chrono")]
            Value::DateTime(dt) => duckdb::types::Value::Timestamp(
                duckdb::types::TimeUnit::Nanosecond,
                dt.timestamp_nanos_opt().unwrap_or(0),
            ),

            #[cfg(feature = "uuid")]
            Value::Uuid(u) => duckdb::types::Value::Text(u.to_string()),
        })
    }
}

impl duckdb::types::ToSql for DuckDbValue {
    fn to_sql(&self) -> duckdb::Result<duckdb::types::ToSqlOutput<'_>> {
        self.0.to_sql()
    }
}
