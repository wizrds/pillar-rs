use arrow::record_batch::RecordBatch;

use crate::{ast::SelectStatement, column::ColumnDef, errors::Error};


/// Describes a materialized view that can be queried through pillar.
///
/// This trait is implemented automatically by the [`MaterializedView`](pillar_macros::MaterializedView) derive macro.
pub trait MaterializedView: Sized + Send + Sync {
    /// The name of the materialized view in the database.
    fn view_name() -> &'static str;

    /// The column definitions for this view.
    fn columns() -> &'static [ColumnDef];

    /// Deserializes a [`RecordBatch`] into a `Vec` of this view type.
    fn from_record_batch(batch: RecordBatch) -> Result<Vec<Self>, Error>;
}

/// Provides a base [`SelectStatement`](crate::ast::SelectStatement) used when creating the materialized view.
///
/// Implemented automatically when `from` is set on the
/// [`MaterializedView`](pillar_macros::MaterializedView) derive macro, or manually
/// for custom query logic.
pub trait ViewQuery: MaterializedView {
    /// The query that defines the contents of this view.
    fn query() -> SelectStatement;
}
