use crate::{
    ast::{
        ColumnRef,
        Statement,
        TableRef,
        UpdateStatement,
    },
    condition::{Condition, ConditionExpression},
    database::Database,
    errors::Error,
    model::Model,
    value::Value,
};


/// A builder for an `UPDATE` statement targeting a [`Model`] table.
#[derive(Debug, Clone)]
pub struct Update<M: Model> {
    statement: UpdateStatement,
    _marker: std::marker::PhantomData<M>,
}

impl<M: Model> Update<M> {
    /// Creates a new [`Update`] for the model's table.
    pub fn new() -> Self {
        Self {
            statement: UpdateStatement::new(TableRef::new(M::table_name())),
            _marker: std::marker::PhantomData,
        }
    }

    /// Appends a column/value assignment.
    pub fn set(mut self, column: impl Into<ColumnRef>, value: impl Into<Value>) -> Self {
        self.statement.set.push((column.into(), value.into()));
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

    /// Clears all column/value assignments.
    pub fn clear_set(mut self) -> Self {
        self.statement = self.statement.clear_set();
        self
    }

    /// Clears the WHERE clause.
    pub fn clear_where(mut self) -> Self {
        self.statement = self.statement.clear_where();
        self
    }

    /// Converts this builder into a [`Statement`].
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

impl<M: Model> Default for Update<M> {
    fn default() -> Self {
        Self::new()
    }
}
