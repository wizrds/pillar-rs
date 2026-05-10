use arrow::record_batch::RecordBatch;

use crate::{
    ast::{Statement, SelectStatement, CreateViewStatement},
    column::ColumnDef,
    errors::Error
};


/// Describes a view that can be queried through pillar.
///
/// This trait is implemented automatically by the [`View`](pillar_macros::View) derive macro.
pub trait View: Sized + Send + Sync {
    /// The name of the view in the database.
    fn view_name() -> &'static str;

    /// The column definitions for this view.
    fn columns() -> &'static [ColumnDef];

    /// Deserializes a [`RecordBatch`] into a `Vec` of this view type.
    fn from_record_batch(batch: RecordBatch) -> Result<Vec<Self>, Error>;
}

/// Provides a base [`SelectStatement`](crate::ast::SelectStatement) used when creating the view.
///
/// Implemented automatically when `from` is set on the
/// [`View`](pillar_macros::View) derive macro, or manually for custom query logic.
pub trait ViewQuery: View {
    /// The query that defines the contents of this view.
    fn query() -> SelectStatement;
}

/// A [`View`] that can produce the DDL statement needed to create its backing view.
pub trait ViewSchema: ViewQuery + Sized {
    /// Returns a [`Statement`] that creates this view.
    fn create_statement() -> Statement {
        Statement::CreateView(
            CreateViewStatement::new(Self::view_name(), Self::query())
                .if_not_exists(),
        )
    }
}
