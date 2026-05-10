use arrow::{
    array::{
        Array,
        BooleanArray,
        Float32Array, Float64Array,
        Int8Array, Int16Array, Int32Array, Int64Array,
        LargeStringArray, StringArray,
        UInt8Array, UInt16Array, UInt32Array, UInt64Array,
    },
    record_batch::RecordBatch,
};

#[cfg(feature = "chrono")]
use arrow::{
    array::{Date32Array, TimestampMicrosecondArray, TimestampNanosecondArray},
    datatypes::{DataType, TimeUnit},
};

#[cfg(feature = "uuid")]
use arrow::array::FixedSizeBinaryArray;

use crate::{errors::Error, value::Value};


/// Extracts a single typed value from a column in an Arrow array at a given row index.
///
/// This is the scalar building block used by the tuple `FromBatch` impls.
pub trait FromArrow: Sized {
    fn from_array(array: &dyn Array, row: usize) -> Option<Self>;
}

/// Deserializes an entire Arrow `RecordBatch` into a `Vec<Self>`.
///
/// Implemented automatically by `#[derive(FromBatch)]` for structs. Tuples up to arity 8
/// have blanket impls using `FromArrow` column-by-column.
pub trait FromBatch: Sized {
    fn from_batch(batch: RecordBatch) -> Result<Vec<Self>, Error>;
}

/// Serializes a value into a row of `Value`s in column order, for use in `INSERT` statements.
///
/// Implemented automatically by `#[derive(ToRow)]` for structs.
pub trait ToRow {
    fn to_row(&self) -> Vec<Value>;
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


macro_rules! impl_from_batch_tuple {
    ( $( ($idx:tt => $T:ident) ),+ ) => {
        impl<$($T),+> FromBatch for ($($T,)+)
        where
            $($T: FromArrow,)+
        {
            fn from_batch(batch: RecordBatch) -> Result<Vec<Self>, Error> {
                (0..batch.num_rows())
                    .map(|row| {
                        Ok((
                            $(
                                $T::from_array(batch.column($idx).as_ref(), row)
                                    .ok_or_else(|| Error::serialization(
                                        format!(
                                            "null or type mismatch at column {} row {}",
                                            $idx,
                                            row,
                                        )
                                    ))?,
                            )+
                        ))
                    })
                    .collect()
            }
        }
    };
}

impl_from_batch_tuple!((0 => A));
impl_from_batch_tuple!((0 => A), (1 => B));
impl_from_batch_tuple!((0 => A), (1 => B), (2 => C));
impl_from_batch_tuple!((0 => A), (1 => B), (2 => C), (3 => D));
impl_from_batch_tuple!((0 => A), (1 => B), (2 => C), (3 => D), (4 => E));
impl_from_batch_tuple!((0 => A), (1 => B), (2 => C), (3 => D), (4 => E), (5 => F));
impl_from_batch_tuple!((0 => A), (1 => B), (2 => C), (3 => D), (4 => E), (5 => F), (6 => G));
impl_from_batch_tuple!((0 => A), (1 => B), (2 => C), (3 => D), (4 => E), (5 => F), (6 => G), (7 => H));


#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use arrow::{
        array::{Int64Array, StringArray},
        datatypes::{DataType, Field, Schema},
        record_batch::RecordBatch,
    };

    use super::FromBatch;

    #[test]
    fn test_tuple_from_batch() {
        let schema = Arc::new(Schema::new(vec![
            Field::new("count", DataType::Int64, false),
            Field::new("name", DataType::Utf8, false),
        ]));

        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(Int64Array::from(vec![1, 2, 3])),
                Arc::new(StringArray::from(vec!["a", "b", "c"])),
            ],
        )
        .unwrap();

        let rows = <(i64, String)>::from_batch(batch).unwrap();

        assert_eq!(rows, vec![
            (1, "a".to_owned()),
            (2, "b".to_owned()),
            (3, "c".to_owned()),
        ]);
    }
}
