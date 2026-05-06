use arrow::record_batch::RecordBatch;

use crate::{column::ColumnDef, errors::Error};


pub trait Model: Sized + Send + Sync {
    fn table_name() -> &'static str;
    fn columns() -> &'static [ColumnDef];
    fn from_record_batch(batch: RecordBatch) -> Result<Vec<Self>, Error>;
    fn to_record_batch(rows: &[Self]) -> Result<RecordBatch, Error>;

    fn primary_keys() -> Vec<&'static str> {
        Self::columns()
            .iter()
            .filter(|col| col.primary_key)
            .map(|col| col.name)
            .collect()
    }

    fn get_column(name: &str) -> Option<&'static ColumnDef> {
        Self::columns()
            .iter()
            .find(|col| col.name == name)
    }
}
