use arrow::record_batch::RecordBatch;

use crate::{column::ColumnDef, errors::Error};


/// Describes a table-backed data type that can be queried and mutated through pillar.
///
/// This trait is implemented automatically by the [`Model`](pillar_macros::Model) derive macro.
pub trait Model: Sized + Send + Sync {
    /// The name of the database table this model maps to.
    fn table_name() -> &'static str;

    /// The column definitions for this model.
    fn columns() -> &'static [ColumnDef];

    /// Deserializes a [`RecordBatch`] into a `Vec` of this model.
    fn from_record_batch(batch: RecordBatch) -> Result<Vec<Self>, Error>;

    /// Serializes a slice of this model into a [`RecordBatch`].
    fn to_record_batch(rows: &[Self]) -> Result<RecordBatch, Error>;

    /// Returns the names of all columns marked as primary keys.
    fn primary_keys() -> Vec<&'static str> {
        Self::columns()
            .iter()
            .filter(|col| col.primary_key)
            .map(|col| col.name)
            .collect()
    }

    /// Returns the [`ColumnDef`](crate::column::ColumnDef) for the column with the given name, if it exists.
    fn get_column(name: &str) -> Option<&'static ColumnDef> {
        Self::columns()
            .iter()
            .find(|col| col.name == name)
    }
}
