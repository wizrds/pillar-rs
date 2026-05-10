use std::marker::PhantomData;

use futures::stream::{Stream, StreamExt};

use crate::{
    ast::{
        AggregateFunction,
        ColumnRef,
        CountArg,
        FromSource,
        Join,
        JoinType,
        OrderBy,
        Projection,
        SelectStatement,
        Statement,
        TableRef,
    },
    condition::{Condition, ConditionExpression},
    convert::FromBatch,
    database::Database,
    errors::Error,
};


/// A builder for a `SELECT` query that deserializes results into `T`.
///
/// `T` can be any type implementing [`FromBatch`]: a model, a view, a plain
/// struct with `#[derive(FromBatch)]`, or a tuple like `(i64, String)`.
pub struct Select<T: FromBatch> {
    statement: SelectStatement,
    _marker: PhantomData<T>,
}

impl<T: FromBatch> Select<T> {
    /// Creates a new [`Select`] reading from the given table or subquery source.
    pub fn new(from: impl Into<FromSource>) -> Self {
        Self {
            statement: SelectStatement::new(from),
            _marker: PhantomData,
        }
    }

    /// Changes the output type to `U` and clears the projection list.
    ///
    /// All filters, joins, and other clauses are preserved. The projection list
    /// is cleared so the caller can build up the correct columns for `U` from
    /// scratch using methods like [`count`](Self::count) or [`columns`](Self::columns).
    pub fn project<U: FromBatch>(self) -> Select<U> {
        Select {
            statement: self.statement.projections([]),
            _marker: PhantomData,
        }
    }

    /// Replaces the projection list with the given columns.
    pub fn columns<I, C>(mut self, columns: I) -> Self
    where
        I: IntoIterator<Item = C>,
        C: Into<ColumnRef>,
    {
        self.statement = self.statement
            .projections(columns.into_iter().map(Projection::column));

        self
    }

    /// Appends a column projection with an alias.
    pub fn column_as<C: Into<ColumnRef>>(mut self, column: C, alias: impl Into<String>) -> Self {
        self.statement = self.statement
            .projection(Projection::column_alias(column, alias));

        self
    }

    /// Applies a [`Condition`] as the WHERE clause.
    pub fn filter(mut self, condition: impl Into<Condition>) -> Self {
        if let Some(expr) = condition.into().to_expression() {
            self.statement = self.statement.where_clause(expr);
        }

        self
    }

    /// Applies a [`ConditionExpression`] directly as the WHERE clause.
    pub fn filter_expr(mut self, expr: ConditionExpression) -> Self {
        self.statement = self.statement.where_clause(expr);
        self
    }

    /// Applies a filter only when `condition` is `true`.
    pub fn filter_if<F>(self, condition: bool, f: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        if condition { f(self) } else { self }
    }

    /// Appends a [`Join`] clause.
    pub fn join(mut self, join: Join) -> Self {
        self.statement = self.statement.join(join);
        self
    }

    /// Appends an `INNER JOIN` on the given table and condition.
    pub fn inner_join(self, table: impl Into<String>, on: ConditionExpression) -> Self {
        self.join(Join {
            table: TableRef::new(table.into()),
            join_type: JoinType::Inner,
            on,
        })
    }

    /// Appends a `LEFT JOIN` on the given table and condition.
    pub fn left_join(self, table: impl Into<String>, on: ConditionExpression) -> Self {
        self.join(Join {
            table: TableRef::new(table.into()),
            join_type: JoinType::Left,
            on,
        })
    }

    /// Sets the `GROUP BY` columns.
    pub fn group_by<I, C>(mut self, columns: I) -> Self
    where
        I: IntoIterator<Item = C>,
        C: Into<ColumnRef>,
    {
        self.statement = self.statement.group_by(columns);
        self
    }

    /// Sets the `HAVING` clause.
    pub fn having(mut self, condition: impl Into<Condition>) -> Self {
        if let Some(expr) = condition.into().to_expression() {
            self.statement = self.statement.having(expr);
        }

        self
    }

    /// Appends an ascending `ORDER BY` on the given column.
    pub fn order_by_asc<C: Into<ColumnRef>>(mut self, column: C) -> Self {
        self.statement = self.statement.order_by(OrderBy::asc(column));
        self
    }

    /// Appends a descending `ORDER BY` on the given column.
    pub fn order_by_desc<C: Into<ColumnRef>>(mut self, column: C) -> Self {
        self.statement = self.statement.order_by(OrderBy::desc(column));
        self
    }

    /// Appends an [`OrderBy`] directive.
    pub fn order_by(mut self, order_by: OrderBy) -> Self {
        self.statement = self.statement.order_by(order_by);
        self
    }

    /// Sets the `LIMIT`.
    pub fn limit(mut self, limit: u64) -> Self {
        self.statement = self.statement.limit(limit);
        self
    }

    /// Sets the `OFFSET`.
    pub fn offset(mut self, offset: u64) -> Self {
        self.statement = self.statement.offset(offset);
        self
    }

    /// Adds `DISTINCT` to the query.
    pub fn distinct(mut self) -> Self {
        self.statement.distinct = true;
        self
    }

    /// Removes `DISTINCT` from the query.
    pub fn clear_distinct(mut self) -> Self {
        self.statement = self.statement.clear_distinct();
        self
    }

    /// Clears the projection list.
    pub fn clear_projections(mut self) -> Self {
        self.statement = self.statement.clear_projections();
        self
    }

    /// Clears the WHERE clause.
    pub fn clear_where(mut self) -> Self {
        self.statement = self.statement.clear_where();
        self
    }

    /// Clears all joins.
    pub fn clear_joins(mut self) -> Self {
        self.statement = self.statement.clear_joins();
        self
    }

    /// Clears the GROUP BY list.
    pub fn clear_group_by(mut self) -> Self {
        self.statement = self.statement.clear_group_by();
        self
    }

    /// Clears the HAVING clause.
    pub fn clear_having(mut self) -> Self {
        self.statement = self.statement.clear_having();
        self
    }

    /// Clears the ORDER BY list.
    pub fn clear_order_by(mut self) -> Self {
        self.statement = self.statement.clear_order_by();
        self
    }

    /// Clears the LIMIT.
    pub fn clear_limit(mut self) -> Self {
        self.statement = self.statement.clear_limit();
        self
    }

    /// Clears the OFFSET.
    pub fn clear_offset(mut self) -> Self {
        self.statement = self.statement.clear_offset();
        self
    }

    /// Clears the WITH clause.
    pub fn clear_with(mut self) -> Self {
        self.statement = self.statement.clear_with();
        self
    }

    /// Appends an aggregate projection.
    pub fn aggregate(mut self, aggregate: AggregateFunction) -> Self {
        self.statement = self.statement.projection(Projection::Aggregate(aggregate));
        self
    }

    /// Appends a `COUNT(*)` aggregate projection.
    pub fn count_all(self) -> Self {
        self.aggregate(AggregateFunction::Count(CountArg::All))
    }

    /// Appends a `COUNT(column)` aggregate projection.
    pub fn count_column<C: Into<ColumnRef>>(self, column: C) -> Self {
        self.aggregate(AggregateFunction::count(column))
    }

    /// Appends a `SUM(column)` aggregate projection.
    pub fn sum<C: Into<ColumnRef>>(self, column: C) -> Self {
        self.aggregate(AggregateFunction::sum(column))
    }

    /// Appends an `AVG(column)` aggregate projection.
    pub fn avg<C: Into<ColumnRef>>(self, column: C) -> Self {
        self.aggregate(AggregateFunction::avg(column))
    }

    /// Appends a `MIN(column)` aggregate projection.
    pub fn min<C: Into<ColumnRef>>(self, column: C) -> Self {
        self.aggregate(AggregateFunction::min(column))
    }

    /// Appends a `MAX(column)` aggregate projection.
    pub fn max<C: Into<ColumnRef>>(self, column: C) -> Self {
        self.aggregate(AggregateFunction::max(column))
    }

    /// Converts this builder into a [`Statement`].
    pub fn into_statement(self) -> Statement {
        Statement::Select(self.statement)
    }

    /// Executes the query and returns all matching rows.
    pub async fn all<D: Database>(self, database: &D) -> Result<Vec<T>, Error> {
        T::from_batch(
            database
                .query(&self.into_statement())
                .await?
                .into_batch(),
        )
    }

    /// Executes the query and returns the first matching row, if any.
    pub async fn one<D: Database>(self, database: &D) -> Result<Option<T>, Error> {
        Ok(
            self.limit(1)
                .all(database)
                .await?
                .pop()
        )
    }

    /// Executes the query with a `COUNT(*)` aggregate and returns the count.
    pub async fn count<D: Database>(self, database: &D) -> Result<u64, Error> {
        let (count,) = self
            .project::<(u64,)>()
            .count_all()
            .clear_order_by()
            .clear_limit()
            .clear_offset()
            .one(database)
            .await?
            .ok_or_else(|| Error::unexpected("Count query returned no rows"))?;

        Ok(count)
    }

    /// Executes the query and returns a stream of row batches.
    pub async fn stream<D: Database>(
        self,
        database: &D,
    ) -> Result<impl Stream<Item = Result<Vec<T>, Error>>, Error> {
        Ok(
            database
                .query_stream(&self.into_statement())
                .await?
                .map(|result| {
                    result
                        .and_then(|qr| T::from_batch(qr.into_batch()))
                        .map_err(Error::from)
                })
        )
    }
}

impl<T: FromBatch> Clone for Select<T> {
    fn clone(&self) -> Self {
        Self {
            statement: self.statement.clone(),
            _marker: PhantomData,
        }
    }
}
