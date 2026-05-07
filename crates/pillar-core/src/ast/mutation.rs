use crate::{
    condition::ConditionExpression,
    value::Value,
    ast::table::TableRef,
};


/// Specifies which columns to target and what to do on a conflict in an [`InsertStatement`](crate::ast::InsertStatement).
#[derive(Debug, Clone, PartialEq)]
pub struct OnConflict {
    pub target: Vec<String>,
    pub action: OnConflictAction,
}

/// The action to take when an [`OnConflict`](crate::ast::OnConflict) condition is met.
#[derive(Debug, Clone, PartialEq)]
pub enum OnConflictAction {
    DoNothing,
    DoUpdate {
        set: Vec<(String, Value)>,
        where_clause: Option<ConditionExpression>,
    },
}

/// AST node for an `INSERT` statement.
#[derive(Debug, Clone, PartialEq)]
pub struct InsertStatement {
    pub table: TableRef,
    pub columns: Vec<String>,
    pub values: Vec<Vec<Value>>,
    pub on_conflict: Option<OnConflict>,
}

impl InsertStatement {
    /// Creates a new [`InsertStatement`](crate::ast::InsertStatement) targeting the given table.
    pub fn new(table: TableRef) -> Self {
        Self { table, columns: Vec::new(), values: Vec::new(), on_conflict: None }
    }

    /// Sets the column names for the insert.
    pub fn columns(mut self, columns: Vec<String>) -> Self {
        self.columns = columns;
        self
    }

    /// Sets the rows of values to insert.
    pub fn values(mut self, values: Vec<Vec<Value>>) -> Self {
        self.values = values;
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
    pub set: Vec<(String, Value)>,
    pub where_clause: Option<ConditionExpression>,
}

impl UpdateStatement {
    /// Creates a new [`UpdateStatement`](crate::ast::UpdateStatement) targeting the given table.
    pub fn new(table: TableRef) -> Self {
        Self { table, set: Vec::new(), where_clause: None }
    }

    /// Sets the column/value pairs to update.
    pub fn set(mut self, set: Vec<(String, Value)>) -> Self {
        self.set = set;
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
    pub fn new(table: TableRef) -> Self {
        Self { table, where_clause: None }
    }

    /// Sets the WHERE clause.
    pub fn where_clause(mut self, condition: ConditionExpression) -> Self {
        self.where_clause = Some(condition);
        self
    }
}
