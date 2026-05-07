use std::sync::Arc;
use arrow::{
    array::ArrayRef,
    compute::cast,
    datatypes::{DataType, Field, Schema},
    record_batch::RecordBatch,
};
use pillar_core::errors::Error;


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
            normalizers: vec![Box::new(TimestampTzNormalizer)],
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


/// Casts any `Timestamp(_, Some(non-UTC tz))` to `Timestamp(_, Some("UTC"))`.
///
/// DuckDB stores timestamps as UTC internally; the timezone on the Arrow type is metadata only.
struct TimestampTzNormalizer;

impl ColumnNormalizer for TimestampTzNormalizer {
    fn applies(&self, data_type: &DataType) -> bool {
        matches!(data_type, DataType::Timestamp(_, Some(tz)) if tz.as_ref() != "UTC")
    }

    fn normalize(&self, field: &Field, array: &ArrayRef) -> Result<(Field, ArrayRef), Error> {
        let DataType::Timestamp(unit, _) = field.data_type() else {
            unreachable!()
        };
        let target = DataType::Timestamp(unit.clone(), Some("UTC".into()));

        Ok((
            Field::new(field.name(), target.clone(), field.is_nullable()),
            cast(array, &target).map_err(|e| Error::serialization(e.to_string()))?,
        ))
    }
}
