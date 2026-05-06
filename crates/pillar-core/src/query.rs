use futures::stream::{Stream, StreamExt};

use crate::{
    errors::Error,
    ast::{
        Statement,
        TableRef,
        Join,
        JoinType,
        OrderBy,
        Projection,
        AggregateFunction,
        CountArg,
        SelectStatement,
        InsertStatement,
    },
    column::IntoColumnRef,
    condition::{Condition, ConditionExpression},
    traits::{Database, Model},
    value::Value,
};


pub struct Select<M: Model> {
    statement: SelectStatement,
    _marker: std::marker::PhantomData<M>,
}

impl<M: Model> Select<M> {
    pub fn new() -> Self {
        Self {
            statement: SelectStatement::new(TableRef::new(M::table_name())),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn columns<I, C>(mut self, columns: I) -> Self
    where
        I: IntoIterator<Item = C>,
        C: IntoColumnRef,
    {
        self.statement = self.statement
            .projections(
                columns
                    .into_iter()
                    .map(|column| Projection::Column(column.into_column_ref()))
                    .collect()
            );
        self
    }

    pub fn column_as<C: IntoColumnRef>(
        mut self,
        column: C,
        alias: impl Into<String>,
    ) -> Self {
        self.statement = self.statement
            .projection(
                Projection::ColumnAlias(
                    column.into_column_ref(),
                    alias.into(),
                )
            );
        self
    }

    pub fn filter(mut self, condition: impl Into<Condition>) -> Self {
        if let Some(expr) = condition
            .into()
            .to_expression()
        {
            self.statement = self.statement.where_clause(expr);
        }
        self
    }

    pub fn filter_expr(mut self, expr: ConditionExpression) -> Self {
        self.statement = self.statement.where_clause(expr);
        self
    }

    pub fn filter_if<F>(self, condition: bool, f: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        if condition {
            f(self)
        } else {
            self
        }
    }

    pub fn join(mut self, join: Join) -> Self {
        self.statement = self.statement.join(join);
        self
    }

    pub fn inner_join(self, table: impl Into<String>, on: ConditionExpression) -> Self {
        self.join(Join {
            table: TableRef::new(table.into()),
            join_type: JoinType::Inner,
            on,
        })
    }

    pub fn left_join(self, table: impl Into<String>, on: ConditionExpression) -> Self {
        self.join(Join {
            table: TableRef::new(table.into()),
            join_type: JoinType::Left,
            on,
        })
    }

    pub fn group_by<I, C>(mut self, columns: I) -> Self
    where
        I: IntoIterator<Item = C>,
        C: IntoColumnRef,
    {
        self.statement = self.statement.group_by(
            columns
                .into_iter()
                .map(|column| column.into_column_ref())
                .collect()
        );
        self
    }

    pub fn having(mut self, condition: impl Into<Condition>) -> Self {
        if let Some(expr) = condition
            .into()
            .to_expression()
        {
            self.statement = self.statement.having(expr);
        }
        self
    }

    pub fn order_by_asc<C: IntoColumnRef>(mut self, column: C) -> Self {
        self.statement = self.statement.order_by_column(OrderBy::asc(column.into_column_ref()));
        self
    }

    pub fn order_by_desc<C: IntoColumnRef>(mut self, column: C) -> Self {
        self.statement = self.statement.order_by_column(OrderBy::desc(column.into_column_ref()));
        self
    }

    pub fn order_by(mut self, order_by: OrderBy) -> Self {
        self.statement = self.statement.order_by_column(order_by);
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

    pub fn distinct(mut self) -> Self {
        self.statement.distinct = true;
        self
    }

    pub fn aggregate(mut self, aggregate: AggregateFunction) -> Self {
        self.statement = self.statement.projection(Projection::Aggregate(aggregate));
        self
    }

    pub fn count(self) -> Self {
        self.aggregate(AggregateFunction::Count(CountArg::All))
    }

    pub fn count_column<C: IntoColumnRef>(self, column: C) -> Self {
        self.aggregate(AggregateFunction::Count(CountArg::Column(column.into_column_ref())))
    }

    pub fn sum<C: IntoColumnRef>(self, column: C) -> Self {
        self.aggregate(AggregateFunction::Sum(column.into_column_ref()))
    }

    pub fn avg<C: IntoColumnRef>(self, column: C) -> Self {
        self.aggregate(AggregateFunction::Avg(column.into_column_ref()))
    }

    pub fn min<C: IntoColumnRef>(self, column: C) -> Self {
        self.aggregate(AggregateFunction::Min(column.into_column_ref()))
    }

    pub fn max<C: IntoColumnRef>(self, column: C) -> Self {
        self.aggregate(AggregateFunction::Max(column.into_column_ref()))
    }

    pub fn into_statement(self) -> Statement {
        Statement::Select(self.statement)
    }

    pub async fn all<D: Database>(self, database: &D) -> Result<Vec<M>, Error> {
        M::from_record_batch(
            database
                .query(&self.into_statement())
                .await?
        )
    }

    pub async fn one<D: Database>(self, database: &D) -> Result<Option<M>, Error> {
        Ok(
            self.limit(1)
                .all(database)
                .await?
                .pop()
        )
    }

    pub async fn stream<D: Database>(self, database: &D) -> Result<impl Stream<Item = Result<Vec<M>, Error>>, Error> {
        Ok(
            database
                .query_stream(&self.into_statement())
                .await?
                .map(|batch_result| {
                    batch_result
                        .and_then(|batch| M::from_record_batch(batch))
                        .map_err(Error::from)
                })
        )
    }
}

impl<M: Model> Default for Select<M> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Insert<M: Model> {
    statement: InsertStatement,
    _marker: std::marker::PhantomData<M>,
}

impl<M: Model> Insert<M> {
    pub fn new(models: Vec<M>) -> Result<Self, Error> {
        if models.is_empty() {
            return Err(Error::invalid_query("Cannot insert empty batch"))
        }

        Ok(Self {
            statement: InsertStatement::new(TableRef::new(M::table_name()))
                .columns(
                    M::columns()
                        .iter()
                        .map(|column| column.name.to_string())
                        .collect()
                ),
                // Get values from record batch after using M::to_record_batch
                // .values(...)
            _marker: std::marker::PhantomData,
        })
    }

    pub fn one(model: M) -> Result<Self, Error> {
        Self::new(vec![model])
    }

    pub fn into_statement(self) -> Statement {
        Statement::Insert(self.statement)
    }

    pub async fn execute<D: Database>(self, database: &D) -> Result<usize, Error> {
        Ok(
            database
                .execute(&self.into_statement())
                .await?
                .rows_affected
        )
    }
}






