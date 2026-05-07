use futures::stream::{Stream, StreamExt};

use crate::{
    errors::Error,
    ast::{
        ColumnDefinition,
        CreateTableStatement,
        DeleteStatement,
        InsertStatement,
        Join,
        JoinType,
        OrderBy,
        Projection,
        AggregateFunction,
        CountArg,
        SelectStatement,
        Statement,
        TableRef,
        UpdateStatement,
    },
    column::IntoColumnRef,
    condition::{Condition, ConditionExpression},
    database::Database,
    model::Model,
    value::Value,
};


/// A builder for a `SELECT` query targeting a [`Model`](crate::model::Model) table.
pub struct SelectEntity<M: Model> {
    statement: SelectStatement,
    _marker: std::marker::PhantomData<M>,
}

impl<M: Model> SelectEntity<M> {
    /// Creates a new [`SelectEntity`](crate::query::SelectEntity) selecting all columns.
    pub fn new() -> Self {
        Self {
            statement: SelectStatement::new(TableRef::new(M::table_name())),
            _marker: std::marker::PhantomData,
        }
    }

    /// Replaces the projection list with the given columns.
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

    /// Appends a column projection with an alias.
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

    /// Applies a [`Condition`](crate::condition::Condition) as the WHERE clause.
    pub fn filter(mut self, condition: impl Into<Condition>) -> Self {
        if let Some(expr) = condition
            .into()
            .to_expression()
        {
            self.statement = self.statement.where_clause(expr);
        }
        self
    }

    /// Applies a [`ConditionExpression`](crate::condition::ConditionExpression) directly as the WHERE clause.
    pub fn filter_expr(mut self, expr: ConditionExpression) -> Self {
        self.statement = self.statement.where_clause(expr);
        self
    }

    /// Applies a filter only when `condition` is true.
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

    /// Appends a [`Join`](crate::ast::Join) clause.
    pub fn join(mut self, join: Join) -> Self {
        self.statement = self.statement.join(join);
        self
    }

    /// Appends an INNER JOIN on the given table and condition.
    pub fn inner_join(self, table: impl Into<String>, on: ConditionExpression) -> Self {
        self.join(Join {
            table: TableRef::new(table.into()),
            join_type: JoinType::Inner,
            on,
        })
    }

    /// Appends a LEFT JOIN on the given table and condition.
    pub fn left_join(self, table: impl Into<String>, on: ConditionExpression) -> Self {
        self.join(Join {
            table: TableRef::new(table.into()),
            join_type: JoinType::Left,
            on,
        })
    }

    /// Sets the GROUP BY columns.
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

    /// Sets the HAVING clause.
    pub fn having(mut self, condition: impl Into<Condition>) -> Self {
        if let Some(expr) = condition
            .into()
            .to_expression()
        {
            self.statement = self.statement.having(expr);
        }
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

    /// Adds `DISTINCT` to the query.
    pub fn distinct(mut self) -> Self {
        self.statement.distinct = true;
        self
    }

    /// Appends an aggregate projection.
    pub fn aggregate(mut self, aggregate: AggregateFunction) -> Self {
        self.statement = self.statement.projection(Projection::Aggregate(aggregate));
        self
    }

    /// Appends a `COUNT(*)` aggregate projection.
    pub fn count(self) -> Self {
        self.aggregate(AggregateFunction::Count(CountArg::All))
    }

    /// Appends a `COUNT(column)` aggregate projection.
    pub fn count_column<C: IntoColumnRef>(self, column: C) -> Self {
        self.aggregate(AggregateFunction::Count(CountArg::Column(column.into_column_ref())))
    }

    /// Appends a `SUM(column)` aggregate projection.
    pub fn sum<C: IntoColumnRef>(self, column: C) -> Self {
        self.aggregate(AggregateFunction::Sum(column.into_column_ref()))
    }

    /// Appends an `AVG(column)` aggregate projection.
    pub fn avg<C: IntoColumnRef>(self, column: C) -> Self {
        self.aggregate(AggregateFunction::Avg(column.into_column_ref()))
    }

    /// Appends a `MIN(column)` aggregate projection.
    pub fn min<C: IntoColumnRef>(self, column: C) -> Self {
        self.aggregate(AggregateFunction::Min(column.into_column_ref()))
    }

    /// Appends a `MAX(column)` aggregate projection.
    pub fn max<C: IntoColumnRef>(self, column: C) -> Self {
        self.aggregate(AggregateFunction::Max(column.into_column_ref()))
    }

    /// Converts this builder into a [`Statement`](crate::ast::Statement).
    pub fn into_statement(self) -> Statement {
        Statement::Select(self.statement)
    }

    /// Executes the query and returns all matching rows.
    pub async fn all<D: Database>(self, database: &D) -> Result<Vec<M>, Error> {
        M::from_record_batch(
            database
                .query(&self.into_statement())
                .await?
                .into_batch()
        )
    }

    /// Executes the query and returns the first matching row, if any.
    pub async fn one<D: Database>(self, database: &D) -> Result<Option<M>, Error> {
        Ok(
            self.limit(1)
                .all(database)
                .await?
                .pop()
        )
    }

    /// Executes the query and returns a stream of row batches.
    pub async fn stream<D: Database>(self, database: &D) -> Result<impl Stream<Item = Result<Vec<M>, Error>>, Error> {
        Ok(
            database
                .query_stream(&self.into_statement())
                .await?
                .map(|batch_result| {
                    batch_result
                        .and_then(|r| M::from_record_batch(r.into_batch()))
                        .map_err(Error::from)
                })
        )
    }
}

impl<M: Model> Default for SelectEntity<M> {
    fn default() -> Self {
        Self::new()
    }
}


/// A builder for an `INSERT` statement targeting a [`Model`](crate::model::Model) table.
pub struct InsertEntity<M: Model> {
    statement: InsertStatement,
    _marker: std::marker::PhantomData<M>,
}

impl<M: Model> InsertEntity<M> {
    /// Creates an insert for a batch of models.
    pub fn many(models: Vec<M>) -> Result<Self, Error> {
        if models.is_empty() {
            return Err(Error::invalid_query("Cannot insert empty batch"))
        }

        Ok(Self {
            statement: InsertStatement::new(TableRef::new(M::table_name()))
                .columns(
                    M::columns()
                        .iter()
                        .map(|col| col.name.to_string())
                        .collect()
                )
                .values(
                    models.iter().map(|m| m.to_row()).collect()
                ),
            _marker: std::marker::PhantomData,
        })
    }

    /// Creates an insert for a single model.
    pub fn one(model: M) -> Result<Self, Error> {
        Self::many(vec![model])
    }

    /// Converts this builder into a [`Statement`](crate::ast::Statement).
    pub fn into_statement(self) -> Statement {
        Statement::Insert(self.statement)
    }

    /// Executes the insert and returns the number of rows affected.
    pub async fn execute<D: Database>(self, database: &D) -> Result<usize, Error> {
        Ok(
            database
                .execute(&self.into_statement())
                .await?
                .rows_affected
        )
    }
}


/// A builder for an `UPDATE` statement targeting a [`Model`](crate::model::Model) table.
pub struct UpdateEntity<M: Model> {
    statement: UpdateStatement,
    _marker: std::marker::PhantomData<M>,
}

impl<M: Model> UpdateEntity<M> {
    /// Creates a new [`UpdateEntity`](crate::query::UpdateEntity) for the model's table.
    pub fn new() -> Self {
        Self {
            statement: UpdateStatement::new(TableRef::new(M::table_name())),
            _marker: std::marker::PhantomData,
        }
    }

    /// Appends a column/value assignment.
    pub fn set(mut self, column: impl Into<String>, value: impl Into<Value>) -> Self {
        self.statement.set.push((column.into(), value.into()));
        self
    }

    /// Applies a [`Condition`](crate::condition::Condition) as the WHERE clause.
    pub fn filter(mut self, condition: impl Into<Condition>) -> Self {
        if let Some(expr) = condition
            .into()
            .to_expression()
        {
            self.statement = self.statement.where_clause(expr);
        }
        self
    }

    /// Applies a [`ConditionExpression`](crate::condition::ConditionExpression) directly as the WHERE clause.
    pub fn filter_expr(mut self, expr: ConditionExpression) -> Self {
        self.statement = self.statement.where_clause(expr);
        self
    }

    /// Converts this builder into a [`Statement`](crate::ast::Statement).
    pub fn into_statement(self) -> Statement {
        Statement::Update(self.statement)
    }

    /// Executes the update and returns the number of rows affected.
    pub async fn execute<D: Database>(self, database: &D) -> Result<usize, Error> {
        Ok(
            database
                .execute(&self.into_statement())
                .await?
                .rows_affected
        )
    }
}

impl<M: Model> Default for UpdateEntity<M> {
    fn default() -> Self {
        Self::new()
    }
}


/// Marker type indicating a [`DeleteEntity`](crate::query::DeleteEntity) has a WHERE clause or explicit all-rows intent.
pub struct Filtered;
/// Marker type indicating a [`DeleteEntity`](crate::query::DeleteEntity) has not yet been given a filter.
pub struct Unfiltered;

/// A builder for a `DELETE` statement targeting a [`Model`](crate::model::Model) table.
///
/// Requires an explicit filter or `all()` call before execution, enforced at compile time via the
/// `S` type parameter.
pub struct DeleteEntity<M: Model, S = Unfiltered> {
    statement: DeleteStatement,
    _marker: std::marker::PhantomData<(M, S)>,
}

impl<M: Model> DeleteEntity<M, Unfiltered> {
    /// Creates a new unfiltered [`DeleteEntity`](crate::query::DeleteEntity).
    pub fn new() -> Self {
        Self {
            statement: DeleteStatement::new(TableRef::new(M::table_name())),
            _marker: std::marker::PhantomData,
        }
    }

    /// Creates a delete that targets all rows in the table.
    pub fn all() -> DeleteEntity<M, Filtered> {
        DeleteEntity {
            statement: DeleteStatement::new(TableRef::new(M::table_name())),
            _marker: std::marker::PhantomData,
        }
    }

    /// Applies a [`Condition`](crate::condition::Condition) as the WHERE clause.
    pub fn filter(self, condition: impl Into<Condition>) -> DeleteEntity<M, Filtered> {
        DeleteEntity {
            statement: match condition.into().to_expression() {
                Some(expr) => self.statement.where_clause(expr),
                None => self.statement,
            },
            _marker: std::marker::PhantomData,
        }
    }

    /// Applies a [`ConditionExpression`](crate::condition::ConditionExpression) directly as the WHERE clause.
    pub fn filter_expr(self, expr: ConditionExpression) -> DeleteEntity<M, Filtered> {
        DeleteEntity {
            statement: self.statement.where_clause(expr),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<M: Model> Default for DeleteEntity<M, Unfiltered> {
    fn default() -> Self {
        Self::new()
    }
}

impl<M: Model> DeleteEntity<M, Filtered> {
    /// Converts this builder into a [`Statement`](crate::ast::Statement).
    pub fn into_statement(self) -> Statement {
        Statement::Delete(self.statement)
    }

    /// Executes the delete and returns the number of rows affected.
    pub async fn execute<D: Database>(self, database: &D) -> Result<usize, Error> {
        Ok(
            database
                .execute(&self.into_statement())
                .await?
                .rows_affected
        )
    }
}

/// Provides query and mutation entry points for any type implementing [`Model`](crate::model::Model).
pub trait EntityOps: Model + Sized {
    /// Returns a new [`SelectEntity`](crate::query::SelectEntity) for this model.
    fn find() -> SelectEntity<Self> {
        SelectEntity::new()
    }

    /// Returns an [`InsertEntity`](crate::query::InsertEntity) for a single model instance.
    fn insert(model: Self) -> Result<InsertEntity<Self>, Error> {
        InsertEntity::one(model)
    }

    /// Returns an [`InsertEntity`](crate::query::InsertEntity) for a batch of model instances.
    fn insert_batch(models: Vec<Self>) -> Result<InsertEntity<Self>, Error> {
        InsertEntity::many(models)
    }

    /// Returns a new [`UpdateEntity`](crate::query::UpdateEntity) for this model.
    fn update() -> UpdateEntity<Self> {
        UpdateEntity::new()
    }

    /// Returns a new unfiltered [`DeleteEntity`](crate::query::DeleteEntity) for this model.
    fn delete() -> DeleteEntity<Self, Unfiltered> {
        DeleteEntity::new()
    }

    /// Returns a [`DeleteEntity`](crate::query::DeleteEntity) that targets all rows in the table.
    fn delete_all() -> DeleteEntity<Self, Filtered> {
        DeleteEntity::all()
    }
}

impl<M: Model> EntityOps for M {}

/// A [`Model`](crate::model::Model) that can produce the DDL statement needed to create its backing table.
pub trait TableSchema: Model {
    /// Returns a [`Statement`](crate::ast::Statement) that creates the backing table for this model.
    fn create_statement() -> Statement {
        Statement::CreateTable(
            CreateTableStatement::new(Self::table_name())
                .if_not_exists()
                .columns(
                    Self::columns()
                        .iter()
                        .map(ColumnDefinition::from)
                        .collect(),
                ),
        )
    }
}


