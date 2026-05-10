use std::sync::Arc;
use arrow::{
    datatypes::{Schema, SchemaRef},
    record_batch::RecordBatch,
};

pub use crate::convert::FromArrow;


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
