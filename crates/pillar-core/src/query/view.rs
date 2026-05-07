use futures::stream::{Stream, StreamExt};

use crate::{
    errors::Error,
    ast::{
        CreateMaterializedViewStatement,
        OrderBy,
        Projection,
        SelectStatement,
        Statement,
        TableRef,
    },
    column::IntoColumnRef,
    condition::{Condition, ConditionExpression},
    database::Database,
    view::{MaterializedView, ViewQuery},
};


/// A builder for a `SELECT` query targeting a [`MaterializedView`](crate::view::MaterializedView).
pub struct SelectView<V: MaterializedView> {
    statement: SelectStatement,
    _marker: std::marker::PhantomData<V>,
}

impl<V: MaterializedView> SelectView<V> {
    /// Creates a new [`SelectView`](crate::query::SelectView) selecting all columns.
    pub fn new() -> Self {
        Self {
            statement: SelectStatement::new(TableRef::new(V::view_name())),
            _marker: std::marker::PhantomData,
        }
    }

    /// Replaces the projection list with the given columns.
    pub fn columns<I, C>(mut self, columns: I) -> Self
    where
        I: IntoIterator<Item = C>,
        C: IntoColumnRef,
    {
        self.statement = self.statement.projections(
            columns
                .into_iter()
                .map(|c| Projection::Column(c.into_column_ref())),
        );
        self
    }

    /// Applies a [`Condition`](crate::condition::Condition) as the WHERE clause.
    pub fn filter(mut self, condition: impl Into<Condition>) -> Self {
        if let Some(expr) = condition.into().to_expression() {
            self.statement = self.statement.where_clause(expr);
        }
        self
    }

    /// Applies a [`ConditionExpression`](crate::condition::ConditionExpression) directly as the WHERE clause.
    pub fn filter_expr(mut self, expr: ConditionExpression) -> Self {
        self.statement = self.statement.where_clause(expr);
        self
    }

    /// Appends an ascending ORDER BY on the given column.
    pub fn order_by_asc<C: IntoColumnRef>(mut self, column: C) -> Self {
        self.statement = self.statement.order_by(OrderBy::asc(column.into_column_ref()));
        self
    }

    /// Appends a descending ORDER BY on the given column.
    pub fn order_by_desc<C: IntoColumnRef>(mut self, column: C) -> Self {
        self.statement = self.statement.order_by(OrderBy::desc(column.into_column_ref()));
        self
    }

    /// Appends an [`OrderBy`](crate::ast::OrderBy) directive.
    pub fn order_by(mut self, order_by: OrderBy) -> Self {
        self.statement = self.statement.order_by(order_by);
        self
    }

    /// Sets the LIMIT.
    pub fn limit(mut self, limit: u64) -> Self {
        self.statement = self.statement.limit(limit);
        self
    }

    /// Sets the OFFSET.
    pub fn offset(mut self, offset: u64) -> Self {
        self.statement = self.statement.offset(offset);
        self
    }

    /// Converts this builder into a [`Statement`](crate::ast::Statement).
    pub fn into_statement(self) -> Statement {
        Statement::Select(self.statement)
    }

    /// Executes the query and returns all matching rows.
    pub async fn all<D: Database>(self, database: &D) -> Result<Vec<V>, Error> {
        V::from_record_batch(
            database
                .query(&self.into_statement())
                .await?
                .into_batch()
        )
    }

    /// Executes the query and returns the first matching row, if any.
    pub async fn one<D: Database>(self, database: &D) -> Result<Option<V>, Error> {
        Ok(self.limit(1).all(database).await?.pop())
    }

    /// Executes the query and returns a stream of row batches.
    pub async fn stream<D: Database>(
        self,
        database: &D,
    ) -> Result<impl Stream<Item = Result<Vec<V>, Error>>, Error> {
        Ok(
            database
                .query_stream(&self.into_statement())
                .await?
                .map(|r| r.and_then(|qr| V::from_record_batch(qr.into_batch())).map_err(Error::from))
        )
    }
}

impl<V: MaterializedView> Default for SelectView<V> {
    fn default() -> Self {
        Self::new()
    }
}

/// Provides a `find` entry point for any type implementing [`MaterializedView`](crate::view::MaterializedView).
pub trait ViewOps: MaterializedView + Sized {
    /// Returns a new [`SelectView`](crate::query::SelectView) for this view.
    fn find() -> SelectView<Self> {
        SelectView::new()
    }
}

impl<V: MaterializedView> ViewOps for V {}

/// A [`MaterializedView`](crate::view::MaterializedView) that can produce the DDL statement needed to create itself.
pub trait ViewSchema: ViewQuery + Sized {
    /// Returns a [`Statement`](crate::ast::Statement) that creates this materialized view.
    fn create_statement() -> Statement {
        Statement::CreateMaterializedView(
            CreateMaterializedViewStatement::new(Self::view_name(), Self::query())
                .if_not_exists(),
        )
    }
}

