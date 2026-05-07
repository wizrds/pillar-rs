use crate::{
    condition::ConditionExpression,
    value::Value,
    ast::refs::{ColumnRef, TableRef},
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

/// AST node for an `INSERT` statement.
#[derive(Debug, Clone, PartialEq)]
pub struct InsertStatement {
    pub table: TableRef,
    pub columns: Vec<ColumnRef>,
    pub values: Vec<Vec<Value>>,
    pub on_conflict: Option<OnConflict>,
}

impl InsertStatement {
    /// Creates a new [`InsertStatement`](crate::ast::InsertStatement) targeting the given table.
    pub fn new(table: impl Into<TableRef>) -> Self {
        Self { table: table.into(), columns: Vec::new(), values: Vec::new(), on_conflict: None }
    }

    /// Sets the column names for the insert.
    pub fn columns(mut self, columns: impl IntoIterator<Item = impl Into<ColumnRef>>) -> Self {
        self.columns = columns.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the rows of values to insert.
    pub fn values<R, I, V>(mut self, rows: R) -> Self
    where
        R: IntoIterator<Item = I>,
        I: IntoIterator<Item = V>,
        V: Into<Value>,
    {
        self.values = rows
            .into_iter()
            .map(|row| row.into_iter().map(Into::into).collect())
            .collect();
        self
    }

    /// Sets the conflict resolution strategy.
    pub fn on_conflict(mut self, on_conflict: OnConflict) -> Self {
        self.on_conflict = Some(on_conflict);
        self
    }
}

/// AST node for an `UPDATE` statement.
#[derive(Debug, Clone, PartialEq)]
pub struct UpdateStatement {
    pub table: TableRef,
    pub set: Vec<(ColumnRef, Value)>,
    pub where_clause: Option<ConditionExpression>,
}

impl UpdateStatement {
    /// Creates a new [`UpdateStatement`](crate::ast::UpdateStatement) targeting the given table.
    pub fn new(table: impl Into<TableRef>) -> Self {
        Self { table: table.into(), set: Vec::new(), where_clause: None }
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
}

/// AST node for a `DELETE` statement.
#[derive(Debug, Clone, PartialEq)]
pub struct DeleteStatement {
    pub table: TableRef,
    pub where_clause: Option<ConditionExpression>,
}

impl DeleteStatement {
    /// Creates a new [`DeleteStatement`](crate::ast::DeleteStatement) targeting the given table.
    pub fn new(table: impl Into<TableRef>) -> Self {
        Self { table: table.into(), where_clause: None }
    }

    /// Sets the WHERE clause.
    pub fn where_clause(mut self, condition: ConditionExpression) -> Self {
        self.where_clause = Some(condition);
        self
    }
}
