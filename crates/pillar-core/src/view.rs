use arrow::record_batch::RecordBatch;

use crate::{column::ColumnDef, errors::Error};


pub trait MaterializedView: Sized + Send + Sync {
    fn view_name() -> &'static str;
    fn columns() -> &'static [ColumnDef];
    fn from_record_batch(batch: RecordBatch) -> Result<Vec<Self>, Error>;
}
