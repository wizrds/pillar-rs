use std::sync::Arc;
use arrow::{
    array::{
        Array,
        BooleanArray,
        Float32Array, Float64Array,
        Int8Array, Int16Array, Int32Array, Int64Array,
        LargeStringArray, StringArray,
        UInt8Array, UInt16Array, UInt32Array, UInt64Array,
    },
    datatypes::{Schema, SchemaRef},
    record_batch::RecordBatch,
};

#[cfg(feature = "chrono")]
use arrow::{
    array::{Date32Array, TimestampMicrosecondArray, TimestampNanosecondArray},
    datatypes::{DataType, TimeUnit},
};

#[cfg(feature = "uuid")]
use arrow::array::FixedSizeBinaryArray;


/// The result of a query that returns rows, wrapping an Arrow [`RecordBatch`].
pub struct QueryResult {
    batch: RecordBatch,
}

impl QueryResult {
    pub fn empty() -> Self {
        Self {
            batch: RecordBatch::new_empty(Arc::new(Schema::empty())),
        }
    }

    /// Returns the number of rows in the result.
    pub fn num_rows(&self) -> usize {
        self.batch.num_rows()
    }

    /// Returns the number of columns in the result.
    pub fn num_columns(&self) -> usize {
        self.batch.num_columns()
    }

    /// Returns the schema of the result.
    pub fn schema(&self) -> SchemaRef {
        self.batch.schema()
    }

    /// Returns a reference to the underlying [`RecordBatch`].
    pub fn batch(&self) -> &RecordBatch {
        &self.batch
    }

    /// Consumes this result and returns the underlying [`RecordBatch`].
    pub fn into_batch(self) -> RecordBatch {
        self.batch
    }

    /// Returns the value at the given column and row as `T`, or `None` if null or out of bounds.
    pub fn get_as<T: FromArrow>(&self, col: usize, row: usize) -> Option<T> {
        if col >= self.batch.num_columns() || row >= self.batch.num_rows() {
            return None;
        }

        T::from_array(self.batch.column(col).as_ref(), row)
    }
}

impl From<RecordBatch> for QueryResult {
    fn from(batch: RecordBatch) -> Self {
        Self { batch }
    }
}


/// Converts a single cell from an Arrow array column into a Rust type.
pub trait FromArrow: Sized {
    fn from_array(array: &dyn Array, row: usize) -> Option<Self>;
}

impl FromArrow for bool {
    fn from_array(array: &dyn Array, row: usize) -> Option<Self> {
        if array.is_null(row) {
            return None;
        }

        array.as_any().downcast_ref::<BooleanArray>().map(|a| a.value(row))
    }
}

impl FromArrow for i8 {
    fn from_array(array: &dyn Array, row: usize) -> Option<Self> {
        if array.is_null(row) { return None; }
        array.as_any().downcast_ref::<Int8Array>().map(|a| a.value(row))
    }
}

impl FromArrow for i16 {
    fn from_array(array: &dyn Array, row: usize) -> Option<Self> {
        if array.is_null(row) { return None; }
        array.as_any().downcast_ref::<Int16Array>().map(|a| a.value(row))
    }
}

impl FromArrow for i32 {
    fn from_array(array: &dyn Array, row: usize) -> Option<Self> {
        if array.is_null(row) { return None; }
        array.as_any().downcast_ref::<Int32Array>().map(|a| a.value(row))
    }
}

impl FromArrow for i64 {
    fn from_array(array: &dyn Array, row: usize) -> Option<Self> {
        if array.is_null(row) { return None; }
        array.as_any().downcast_ref::<Int64Array>().map(|a| a.value(row))
    }
}

impl FromArrow for u8 {
    fn from_array(array: &dyn Array, row: usize) -> Option<Self> {
        if array.is_null(row) { return None; }
        array.as_any().downcast_ref::<UInt8Array>().map(|a| a.value(row))
    }
}

impl FromArrow for u16 {
    fn from_array(array: &dyn Array, row: usize) -> Option<Self> {
        if array.is_null(row) { return None; }
        array.as_any().downcast_ref::<UInt16Array>().map(|a| a.value(row))
    }
}

impl FromArrow for u32 {
    fn from_array(array: &dyn Array, row: usize) -> Option<Self> {
        if array.is_null(row) { return None; }
        array.as_any().downcast_ref::<UInt32Array>().map(|a| a.value(row))
    }
}

impl FromArrow for u64 {
    fn from_array(array: &dyn Array, row: usize) -> Option<Self> {
        if array.is_null(row) { return None; }
        array.as_any().downcast_ref::<UInt64Array>().map(|a| a.value(row))
    }
}

impl FromArrow for f32 {
    fn from_array(array: &dyn Array, row: usize) -> Option<Self> {
        if array.is_null(row) { return None; }
        array.as_any().downcast_ref::<Float32Array>().map(|a| a.value(row))
    }
}

impl FromArrow for f64 {
    fn from_array(array: &dyn Array, row: usize) -> Option<Self> {
        if array.is_null(row) { return None; }
        array.as_any().downcast_ref::<Float64Array>().map(|a| a.value(row))
    }
}

impl FromArrow for String {
    fn from_array(array: &dyn Array, row: usize) -> Option<Self> {
        if array.is_null(row) { return None; }

        array.as_any().downcast_ref::<StringArray>()
            .map(|a| a.value(row).to_owned())
            .or_else(|| {
                array.as_any().downcast_ref::<LargeStringArray>()
                    .map(|a| a.value(row).to_owned())
            })
    }
}

#[cfg(feature = "chrono")]
impl FromArrow for chrono::NaiveDate {
    fn from_array(array: &dyn Array, row: usize) -> Option<Self> {
        if array.is_null(row) { return None; }

        array.as_any().downcast_ref::<Date32Array>().and_then(|a| {
            chrono::NaiveDate::from_num_days_from_ce_opt(a.value(row) + 719_163)
        })
    }
}

#[cfg(feature = "chrono")]
impl FromArrow for chrono::DateTime<chrono::Utc> {
    fn from_array(array: &dyn Array, row: usize) -> Option<Self> {
        use chrono::TimeZone;

        if array.is_null(row) { return None; }

        if let Some(a) = array.as_any().downcast_ref::<TimestampNanosecondArray>() {
            return chrono::Utc.timestamp_nanos(a.value(row)).into();
        }

        if let Some(a) = array.as_any().downcast_ref::<TimestampMicrosecondArray>() {
            let micros = a.value(row);
            let secs = micros / 1_000_000;
            let nanos = ((micros % 1_000_000) * 1_000) as u32;
            return chrono::Utc.timestamp_opt(secs, nanos).single();
        }

        match array.data_type() {
            DataType::Timestamp(TimeUnit::Nanosecond, _) => {
                array.as_any().downcast_ref::<TimestampNanosecondArray>()
                    .and_then(|a| chrono::Utc.timestamp_nanos(a.value(row)).into())
            }
            _ => None,
        }
    }
}

#[cfg(feature = "uuid")]
impl FromArrow for uuid::Uuid {
    fn from_array(array: &dyn Array, row: usize) -> Option<Self> {
        if array.is_null(row) { return None; }

        array.as_any().downcast_ref::<FixedSizeBinaryArray>()
            .and_then(|a| uuid::Uuid::from_slice(a.value(row)).ok())
    }
}
