use crate::{
    condition::ConditionExpression,
    value::Value,
    ast::{
        refs::{ColumnRef, TableRef},
        select::{Projection, SelectStatement},
    },
};


/// Specifies which columns to target and what to do on a conflict in an [`InsertStatement`](crate::ast::InsertStatement).
#[derive(Debug, Clone, PartialEq)]
pub struct OnConflict {
    pub target: Vec<ColumnRef>,
    pub action: OnConflictAction,
}

impl OnConflict {
    /// Creates a new [`OnConflict`](crate::ast::OnConflict) with the given action and no conflict target columns.
    pub fn new(action: OnConflictAction) -> Self {
        Self { target: Vec::new(), action }
    }

    /// Sets the conflict target columns.
    pub fn target(mut self, columns: impl IntoIterator<Item = impl Into<ColumnRef>>) -> Self {
        self.target = columns.into_iter().map(Into::into).collect();
        self
    }
}

/// The action to take when an [`OnConflict`](crate::ast::OnConflict) condition is met.
#[derive(Debug, Clone, PartialEq)]
pub enum OnConflictAction {
    DoNothing,
    DoUpdate {
        set: Vec<(ColumnRef, Value)>,
        where_clause: Option<ConditionExpression>,
    },
}

impl OnConflictAction {
    /// Creates a `DO NOTHING` action.
    pub fn do_nothing() -> Self {
        Self::DoNothing
    }

    /// Creates a `DO UPDATE SET` action with the given column/value pairs.
    pub fn do_update<I, K, V>(set: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<ColumnRef>,
        V: Into<Value>,
    {
        Self::DoUpdate {
            set: set.into_iter().map(|(k, v)| (k.into(), v.into())).collect(),
            where_clause: None,
        }
    }

    /// Adds a WHERE guard to a `DO UPDATE` action. Has no effect on `DO NOTHING`.
    pub fn where_clause(self, condition: ConditionExpression) -> Self {
        match self {
            Self::DoUpdate { set, .. } => Self::DoUpdate { set, where_clause: Some(condition) },
            other => other,
        }
    }
}

/// The body of an `INSERT` statement: either literal values or a subquery.
#[derive(Debug, Clone, PartialEq)]
pub enum InsertBody {
    /// `VALUES (...)` rows.
    Values(Vec<Vec<Value>>),
    /// `SELECT ...` as the source of rows.
    Select(Box<SelectStatement>),
}

impl InsertBody {
    /// Creates an `InsertBody` with the given rows of literal values.
    pub fn values<R, I, V>(rows: R) -> Self
    where
        R: IntoIterator<Item = I>,
        I: IntoIterator<Item = V>,
        V: Into<Value>,
    {
        Self::Values(
            rows.into_iter()
                .map(|row| row.into_iter().map(Into::into).collect())
                .collect(),
        )
    }

    /// Creates an `InsertBody` sourcing rows from a `SELECT` statement.
    pub fn select(query: SelectStatement) -> Self {
        Self::Select(Box::new(query))
    }
}

/// AST node for an `INSERT` statement.
#[derive(Debug, Clone, PartialEq)]
pub struct InsertStatement {
    pub table: TableRef,
    pub columns: Vec<ColumnRef>,
    pub body: InsertBody,
    pub on_conflict: Option<OnConflict>,
    pub returning: Option<Vec<Projection>>,
}

impl InsertStatement {
    /// Creates a new [`InsertStatement`](crate::ast::InsertStatement) targeting the given table with no rows.
    pub fn new(table: impl Into<TableRef>) -> Self {
        Self {
            table: table.into(),
            columns: Vec::new(),
            body: InsertBody::Values(Vec::new()),
            on_conflict: None,
            returning: None,
        }
    }

    /// Sets the column names for the insert.
    pub fn columns(mut self, columns: impl IntoIterator<Item = impl Into<ColumnRef>>) -> Self {
        self.columns = columns.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the rows of literal values to insert.
    pub fn values<R, I, V>(mut self, rows: R) -> Self
    where
        R: IntoIterator<Item = I>,
        I: IntoIterator<Item = V>,
        V: Into<Value>,
    {
        self.body = InsertBody::values(rows);
        self
    }

    /// Sets a `SELECT` statement as the source of rows to insert.
    pub fn select(mut self, query: SelectStatement) -> Self {
        self.body = InsertBody::select(query);
        self
    }

    /// Sets the conflict resolution strategy.
    pub fn on_conflict(mut self, on_conflict: OnConflict) -> Self {
        self.on_conflict = Some(on_conflict);
        self
    }

    /// Sets the `RETURNING` clause.
    pub fn returning(
        mut self,
        projections: impl IntoIterator<Item = Projection>,
    ) -> Self {
        self.returning = Some(projections.into_iter().collect());
        self
    }
}

/// AST node for an `UPDATE` statement.
#[derive(Debug, Clone, PartialEq)]
pub struct UpdateStatement {
    pub table: TableRef,
    pub set: Vec<(ColumnRef, Value)>,
    pub where_clause: Option<ConditionExpression>,
    pub returning: Option<Vec<Projection>>,
}

impl UpdateStatement {
    /// Creates a new [`UpdateStatement`](crate::ast::UpdateStatement) targeting the given table.
    pub fn new(table: impl Into<TableRef>) -> Self {
        Self { table: table.into(), set: Vec::new(), where_clause: None, returning: None }
    }

    /// Sets the column/value pairs to update.
    pub fn set<I, K, V>(mut self, set: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<ColumnRef>,
        V: Into<Value>,
    {
        self.set = set.into_iter().map(|(k, v)| (k.into(), v.into())).collect();
        self
    }

    /// Sets the WHERE clause.
    pub fn where_clause(mut self, condition: ConditionExpression) -> Self {
        self.where_clause = Some(condition);
        self
    }

    /// Sets the `RETURNING` clause.
    pub fn returning(
        mut self,
        projections: impl IntoIterator<Item = Projection>,
    ) -> Self {
        self.returning = Some(projections.into_iter().collect());
        self
    }
}

/// AST node for a `DELETE` statement.
#[derive(Debug, Clone, PartialEq)]
pub struct DeleteStatement {
    pub table: TableRef,
    pub where_clause: Option<ConditionExpression>,
    pub returning: Option<Vec<Projection>>,
}

impl DeleteStatement {
    /// Creates a new [`DeleteStatement`](crate::ast::DeleteStatement) targeting the given table.
    pub fn new(table: impl Into<TableRef>) -> Self {
        Self { table: table.into(), where_clause: None, returning: None }
    }

    /// Sets the WHERE clause.
    pub fn where_clause(mut self, condition: ConditionExpression) -> Self {
        self.where_clause = Some(condition);
        self
    }

    /// Sets the `RETURNING` clause.
    pub fn returning(
        mut self,
        projections: impl IntoIterator<Item = Projection>,
    ) -> Self {
        self.returning = Some(projections.into_iter().collect());
        self
    }
}
