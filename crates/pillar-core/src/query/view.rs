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


pub struct SelectView<V: MaterializedView> {
    statement: SelectStatement,
    _marker: std::marker::PhantomData<V>,
}

impl<V: MaterializedView> SelectView<V> {
    pub fn new() -> Self {
        Self {
            statement: SelectStatement::new(TableRef::new(V::view_name())),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn columns<I, C>(mut self, columns: I) -> Self
    where
        I: IntoIterator<Item = C>,
        C: IntoColumnRef,
    {
        self.statement = self.statement.projections(
            columns
                .into_iter()
                .map(|c| Projection::Column(c.into_column_ref()))
                .collect(),
        );
        self
    }

    pub fn filter(mut self, condition: impl Into<Condition>) -> Self {
        if let Some(expr) = condition.into().to_expression() {
            self.statement = self.statement.where_clause(expr);
        }
        self
    }

    pub fn filter_expr(mut self, expr: ConditionExpression) -> Self {
        self.statement = self.statement.where_clause(expr);
        self
    }

    pub fn order_by_asc<C: IntoColumnRef>(mut self, column: C) -> Self {
        self.statement = self.statement.order_by(OrderBy::asc(column.into_column_ref()));
        self
    }

    pub fn order_by_desc<C: IntoColumnRef>(mut self, column: C) -> Self {
        self.statement = self.statement.order_by(OrderBy::desc(column.into_column_ref()));
        self
    }

    pub fn order_by(mut self, order_by: OrderBy) -> Self {
        self.statement = self.statement.order_by(order_by);
        self
    }

    pub fn limit(mut self, limit: u64) -> Self {
        self.statement = self.statement.limit(limit);
        self
    }

    pub fn offset(mut self, offset: u64) -> Self {
        self.statement = self.statement.offset(offset);
        self
    }

    pub fn into_statement(self) -> Statement {
        Statement::Select(self.statement)
    }

    pub async fn all<D: Database>(self, database: &D) -> Result<Vec<V>, Error> {
        V::from_record_batch(database.query(&self.into_statement()).await?)
    }

    pub async fn one<D: Database>(self, database: &D) -> Result<Option<V>, Error> {
        Ok(self.limit(1).all(database).await?.pop())
    }

    pub async fn stream<D: Database>(
        self,
        database: &D,
    ) -> Result<impl Stream<Item = Result<Vec<V>, Error>>, Error> {
        Ok(database
            .query_stream(&self.into_statement())
            .await?
            .map(|batch| batch.and_then(|b| V::from_record_batch(b)).map_err(Error::from)))
    }
}

impl<V: MaterializedView> Default for SelectView<V> {
    fn default() -> Self {
        Self::new()
    }
}

pub trait ViewOps: MaterializedView + Sized {
    fn find() -> SelectView<Self> {
        SelectView::new()
    }
}

pub trait DefinedView: ViewQuery + Sized {
    fn create_statement() -> Statement {
        Statement::CreateMaterializedView(
            CreateMaterializedViewStatement::new(Self::view_name(), Self::query()),
        )
    }
}

impl<V: ViewQuery> DefinedView for V {}

impl<V: MaterializedView> ViewOps for V {}
