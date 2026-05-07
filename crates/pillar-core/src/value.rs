use std::fmt::{Display, Formatter, Result as FmtResult};


/// Converts a value to its SQL literal representation.
pub trait ToSql {
    /// Returns the SQL literal string for this value.
    fn to_sql(&self) -> String;
}

/// A runtime value that can be passed as a query parameter or stored as a column default.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Boolean(bool),
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    UInt8(u8),
    UInt16(u16),
    UInt32(u32),
    UInt64(u64),
    Float32(f32),
    Float64(f64),
    String(String),
    Bytes(Vec<u8>),
    List(Vec<Value>),
    Map(Vec<(Value, Value)>),
    #[cfg(feature = "chrono")]
    Date(chrono::NaiveDate),
    #[cfg(feature = "chrono")]
    Time(chrono::NaiveTime),
    #[cfg(feature = "chrono")]
    DateTime(chrono::DateTime<chrono::Utc>),
    #[cfg(feature = "uuid")]
    Uuid(uuid::Uuid),
}

impl Value {
    /// Creates a new [`Value::Null`](crate::value::Value::Null) instance.
    pub fn null() -> Self {
        Value::Null
    }

    /// Creates a new [`Value::Boolean`](crate::value::Value::Boolean) instance.
    pub fn bool(value: impl Into<bool>) -> Self {
        Value::Boolean(value.into())
    }

    /// Creates a new [`Value::Int8`](crate::value::Value::Int8) instance.
    pub fn int8(value: impl Into<i8>) -> Self {
        Value::Int8(value.into())
    }

    /// Creates a new [`Value::Int16`](crate::value::Value::Int16) instance.
    pub fn int16(value: impl Into<i16>) -> Self {
        Value::Int16(value.into())
    }

    /// Creates a new [`Value::Int32`](crate::value::Value::Int32) instance.
    pub fn int32(value: impl Into<i32>) -> Self {
        Value::Int32(value.into())
    }

    /// Creates a new [`Value::Int64`](crate::value::Value::Int64) instance.
    pub fn int64(value: impl Into<i64>) -> Self {
        Value::Int64(value.into())
    }

    /// Creates a new [`Value::UInt8`](crate::value::Value::UInt8) instance.
    pub fn uint8(value: impl Into<u8>) -> Self {
        Value::UInt8(value.into())
    }

    /// Creates a new [`Value::UInt16`](crate::value::Value::UInt16) instance.
    pub fn uint16(value: impl Into<u16>) -> Self {
        Value::UInt16(value.into())
    }

    /// Creates a new [`Value::UInt32`](crate::value::Value::UInt32) instance.
    pub fn uint32(value: impl Into<u32>) -> Self {
        Value::UInt32(value.into())
    }

    /// Creates a new [`Value::UInt64`](crate::value::Value::UInt64) instance.
    pub fn uint64(value: impl Into<u64>) -> Self {
        Value::UInt64(value.into())
    }

    /// Creates a new [`Value::Float32`](crate::value::Value::Float32) instance.
    pub fn float32(value: impl Into<f32>) -> Self {
        Value::Float32(value.into())
    }

    /// Creates a new [`Value::Float64`](crate::value::Value::Float64) instance.
    pub fn float64(value: impl Into<f64>) -> Self {
        Value::Float64(value.into())
    }

    /// Creates a new [`Value::String`](crate::value::Value::String) instance.
    pub fn string(value: impl Into<String>) -> Self {
        Value::String(value.into())
    }

    /// Creates a new [`Value::Bytes`](crate::value::Value::Bytes) instance.
    pub fn bytes(value: impl Into<Vec<u8>>) -> Self {
        Value::Bytes(value.into())
    }

    /// Creates a new [`Value::List`](crate::value::Value::List) instance.
    pub fn list(value: impl IntoIterator<Item = impl Into<Value>>) -> Self {
        Value::List(value.into_iter().map(Into::into).collect())
    }

    /// Creates a new [`Value::Map`](crate::value::Value::Map) instance.
    pub fn map(value: impl IntoIterator<Item = (impl Into<Value>, impl Into<Value>)>) -> Self {
        Value::Map(value.into_iter().map(|(k, v)| (k.into(), v.into())).collect())
    }
    
    #[cfg(feature = "chrono")]
    /// Creates a new [`Value::Date`](crate::value::Value::Date) instance.
    pub fn date(value: impl Into<chrono::NaiveDate>) -> Self {
        Value::Date(value.into())
    }

    #[cfg(feature = "chrono")]
    /// Creates a new [`Value::Time`](crate::value::Value::Time) instance.
    pub fn time(value: impl Into<chrono::NaiveTime>) -> Self {
        Value::Time(value.into())
    }

    #[cfg(feature = "chrono")]
    /// Creates a new [`Value::DateTime`](crate::value::Value::DateTime) instance.
    pub fn datetime(value: impl Into<chrono::DateTime<chrono::Utc>>) -> Self {
        Value::DateTime(value.into())
    }

    #[cfg(feature = "uuid")]
    /// Creates a new [`Value::Uuid`](crate::value::Value::Uuid) instance.
    pub fn uuid(value: impl Into<uuid::Uuid>) -> Self {
        Value::Uuid(value.into())
    }
    
    /// Returns `true` if this value is [`Value::Null`](crate::value::Value::Null).
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }
}

impl ToSql for Value {
    fn to_sql(&self) -> String {
        match self {
            Value::Null => "NULL".to_string(),
            Value::Boolean(b) => b.to_string(),
            Value::Int8(i) => i.to_string(),
            Value::Int16(i) => i.to_string(),
            Value::Int32(i) => i.to_string(),
            Value::Int64(i) => i.to_string(),
            Value::UInt8(u) => u.to_string(),
            Value::UInt16(u) => u.to_string(),
            Value::UInt32(u) => u.to_string(),
            Value::UInt64(u) => u.to_string(),
            Value::Float32(f) => f.to_string(),
            Value::Float64(f) => f.to_string(),
            Value::String(s) => format!("'{}'", s.replace("'", "''")),
            Value::Bytes(b) => format!("X'{}'", hex::encode(b)),
            Value::List(l) => format!(
                "[{}]",
                l.iter()
                    .map(|v| v.to_sql())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Value::Map(m) => format!(
                "{{{}}}",
                m.iter()
                    .map(|(k, v)| format!("{}: {}", k.to_sql(), v.to_sql()))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            #[cfg(feature = "chrono")]
            Value::Date(d) => format!("'{}'", d.format("%Y-%m-%d")),
            #[cfg(feature = "chrono")]
            Value::Time(t) => format!("'{}'", t.format("%H:%M:%S")),
            #[cfg(feature = "chrono")]
            Value::DateTime(dt) => format!("'{}'", dt.to_rfc3339()),
            #[cfg(feature = "uuid")]
            Value::Uuid(u) => format!("'{}'", u.to_string()),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.to_sql())
    }
}

impl From<bool> for Value { fn from(value: bool) -> Self { Value::Boolean(value) } }
impl From<i8> for Value { fn from(value: i8) -> Self { Value::Int8(value) } }
impl From<i16> for Value { fn from(value: i16) -> Self { Value::Int16(value) } }
impl From<i32> for Value { fn from(value: i32) -> Self { Value::Int32(value) } }
impl From<i64> for Value { fn from(value: i64) -> Self { Value::Int64(value) } }
impl From<u8> for Value { fn from(value: u8) -> Self { Value::UInt8(value) } }
impl From<u16> for Value { fn from(value: u16) -> Self { Value::UInt16(value) } }
impl From<u32> for Value { fn from(value: u32) -> Self { Value::UInt32(value) } }
impl From<u64> for Value { fn from(value: u64) -> Self { Value::UInt64(value) } }
impl From<f32> for Value { fn from(value: f32) -> Self { Value::Float32(value) } }
impl From<f64> for Value { fn from(value: f64) -> Self { Value::Float64(value) } }
impl From<String> for Value { fn from(value: String) -> Self { Value::String(value) } }
impl From<&str> for Value { fn from(value: &str) -> Self { Value::String(value.to_string()) } }
impl From<Vec<u8>> for Value { fn from(value: Vec<u8>) -> Self { Value::Bytes(value) } }
impl<T: Into<Value>> From<Option<T>> for Value {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(v) => v.into(),
            None => Value::Null,
        }
    }
}

#[cfg(feature = "chrono")]
impl From<chrono::NaiveDate> for Value { fn from(value: chrono::NaiveDate) -> Self { Value::Date(value) } }
#[cfg(feature = "chrono")]
impl From<chrono::NaiveTime> for Value { fn from(value: chrono::NaiveTime) -> Self { Value::Time(value) } }
#[cfg(feature = "chrono")]
impl From<chrono::DateTime<chrono::Utc>> for Value { fn from(value: chrono::DateTime<chrono::Utc>) -> Self { Value::DateTime(value) } }
#[cfg(feature = "uuid")]
impl From<uuid::Uuid> for Value { fn from(value: uuid::Uuid) -> Self { Value::Uuid(value) } }

impl ToSql for bool { fn to_sql(&self) -> String { Value::Boolean(*self).to_sql() } }
impl ToSql for i8 { fn to_sql(&self) -> String { Value::Int8(*self).to_sql() } }
impl ToSql for i16 { fn to_sql(&self) -> String { Value::Int16(*self).to_sql() } }
impl ToSql for i32 { fn to_sql(&self) -> String { Value::Int32(*self).to_sql() } }
impl ToSql for i64 { fn to_sql(&self) -> String { Value::Int64(*self).to_sql() } }
impl ToSql for u8 { fn to_sql(&self) -> String { Value::UInt8(*self).to_sql() } }
impl ToSql for u16 { fn to_sql(&self) -> String { Value::UInt16(*self).to_sql() } }
impl ToSql for u32 { fn to_sql(&self) -> String { Value::UInt32(*self).to_sql() } }
impl ToSql for u64 { fn to_sql(&self) -> String { Value::UInt64(*self).to_sql() } }
impl ToSql for f32 { fn to_sql(&self) -> String { Value::Float32(*self).to_sql() } }
impl ToSql for f64 { fn to_sql(&self) -> String { Value::Float64(*self).to_sql() } }
impl ToSql for String { fn to_sql(&self) -> String { Value::String(self.clone()).to_sql() } }
impl ToSql for &str { fn to_sql(&self) -> String { Value::String(self.to_string()).to_sql() } }
impl<T: ToSql> ToSql for Option<T> {
    fn to_sql(&self) -> String {
        match self {
            Some(v) => v.to_sql(),
            None => Value::Null.to_sql(),
        }
    }
}

#[cfg(feature = "chrono")]
impl ToSql for chrono::NaiveDate { fn to_sql(&self) -> String { Value::Date(*self).to_sql() } }
#[cfg(feature = "chrono")]
impl ToSql for chrono::NaiveTime { fn to_sql(&self) -> String { Value::Time(*self).to_sql() } }
#[cfg(feature = "chrono")]
impl ToSql for chrono::DateTime<chrono::Utc> { fn to_sql(&self) -> String { Value::DateTime(*self).to_sql() } }
#[cfg(feature = "uuid")]
impl ToSql for uuid::Uuid { fn to_sql(&self) -> String { Value::Uuid(*self).to_sql() } }
