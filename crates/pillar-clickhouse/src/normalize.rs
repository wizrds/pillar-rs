use std::sync::Arc;
use arrow::{
    array::ArrayRef,
    datatypes::{DataType, Field, Schema},
    record_batch::RecordBatch,
};
use pillar_core::errors::Error;

#[cfg(feature = "uuid")]
use arrow::array::{Array, FixedSizeBinaryArray, LargeStringArray};


pub trait ColumnNormalizer: Send + Sync {
    fn applies(&self, data_type: &DataType) -> bool;
    fn normalize(&self, field: &Field, array: &ArrayRef) -> Result<(Field, ArrayRef), Error>;
}


pub struct BatchNormalizer {
    normalizers: Vec<Box<dyn ColumnNormalizer>>,
}

impl BatchNormalizer {
    pub fn new() -> Self {
        Self {
            normalizers: vec![
                #[cfg(feature = "uuid")]
                Box::new(UuidNormalizer),
            ],
        }
    }

    pub fn normalize(&self, batch: RecordBatch) -> Result<RecordBatch, Error> {
        let schema = batch.schema();
        let mut fields = Vec::with_capacity(schema.fields().len());
        let mut columns = Vec::with_capacity(batch.num_columns());

        for (i, field) in schema.fields().iter().enumerate() {
            let (f, c) = self.normalizers
                .iter()
                .find(|n| n.applies(field.data_type()))
                .map(|n| n.normalize(field, batch.column(i)))
                .unwrap_or_else(|| Ok((field.as_ref().clone(), Arc::clone(batch.column(i)))))?;

            fields.push(f);
            columns.push(c);
        }

        RecordBatch::try_new(Arc::new(Schema::new(fields)), columns)
            .map_err(|e| Error::serialization(e.to_string()))
    }
}

impl Default for BatchNormalizer {
    fn default() -> Self {
        Self::new()
    }
}


/// Converts `FixedSizeBinary(16)` columns to `LargeUtf8` by interpreting bytes as UUID.
///
/// ClickHouse returns UUID columns as raw 16-byte binary in Arrow; serde_arrow expects a string.
#[cfg(feature = "uuid")]
struct UuidNormalizer;

#[cfg(feature = "uuid")]
impl ColumnNormalizer for UuidNormalizer {
    fn applies(&self, data_type: &DataType) -> bool {
        matches!(data_type, DataType::FixedSizeBinary(16))
    }

    fn normalize(&self, field: &Field, array: &ArrayRef) -> Result<(Field, ArrayRef), Error> {
        let binary = array
            .as_any()
            .downcast_ref::<FixedSizeBinaryArray>()
            .ok_or_else(|| Error::serialization("expected FixedSizeBinaryArray for UUID column"))?;

        let strings: Result<Vec<Option<String>>, Error> = (0..binary.len())
            .map(|i| {
                if binary.is_null(i) {
                    return Ok(None);
                }

                uuid::Uuid::from_slice(binary.value(i))
                    .map(|u| Some(u.to_string()))
                    .map_err(|e| Error::serialization(e.to_string()))
            })
            .collect();

        Ok((
            Field::new(field.name(), DataType::LargeUtf8, field.is_nullable()),
            Arc::new(LargeStringArray::from(strings?)) as ArrayRef,
        ))
    }
}
