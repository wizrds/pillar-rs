use crate::{
    ast::{
        DeleteStatement,
        Statement,
        TableRef,
    },
    condition::{Condition, ConditionExpression},
    database::Database,
    errors::Error,
    model::Model,
};


/// Marker type: the [`Delete`] builder has a WHERE clause or explicit all-rows intent.
#[derive(Debug, Clone)]
pub struct Filtered;

/// Marker type: the [`Delete`] builder has not yet been given a filter.
#[derive(Debug, Clone)]
pub struct Unfiltered;


/// A builder for a `DELETE` statement targeting a [`Model`] table.
///
/// Requires an explicit filter or [`Delete::all`] before execution,
/// enforced at compile time via the `S` type parameter.
#[derive(Debug, Clone)]
pub struct Delete<M: Model, S = Unfiltered> {
    statement: DeleteStatement,
    _marker: std::marker::PhantomData<(M, S)>,
}

impl<M: Model> Delete<M, Unfiltered> {
    /// Creates a new unfiltered [`Delete`].
    pub fn new() -> Self {
        Self {
            statement: DeleteStatement::new(TableRef::new(M::table_name())),
            _marker: std::marker::PhantomData,
        }
    }

    /// Creates a delete that targets all rows in the table.
    pub fn all() -> Delete<M, Filtered> {
        Delete {
            statement: DeleteStatement::new(TableRef::new(M::table_name())),
            _marker: std::marker::PhantomData,
        }
    }

    /// Applies a [`Condition`] as the WHERE clause.
    pub fn filter(self, condition: impl Into<Condition>) -> Delete<M, Filtered> {
        Delete {
            statement: match condition.into().to_expression() {
                Some(expr) => self.statement.where_clause(expr),
                None => self.statement,
            },
            _marker: std::marker::PhantomData,
        }
    }

    /// Applies a [`ConditionExpression`] directly as the WHERE clause.
    pub fn filter_expr(self, expr: ConditionExpression) -> Delete<M, Filtered> {
        Delete {
            statement: self.statement.where_clause(expr),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<M: Model> Default for Delete<M, Unfiltered> {
    fn default() -> Self {
        Self::new()
    }
}

impl<M: Model> Delete<M, Filtered> {
    /// Converts this builder into a [`Statement`].
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
