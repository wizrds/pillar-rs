use crate::{
    condition::ConditionExpression,
    value::Value,
    ast::table::TableRef,
};


#[derive(Debug, Clone, PartialEq)]
pub struct OnConflict {
    pub target: Vec<String>,
    pub action: OnConflictAction,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OnConflictAction {
    DoNothing,
    DoUpdate {
        set: Vec<(String, Value)>,
        where_clause: Option<ConditionExpression>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct InsertStatement {
    pub table: TableRef,
    pub columns: Vec<String>,
    pub values: Vec<Vec<Value>>,
    pub on_conflict: Option<OnConflict>,
}

impl InsertStatement {
    pub fn new(table: TableRef) -> Self {
        Self { table, columns: Vec::new(), values: Vec::new(), on_conflict: None }
    }

    pub fn columns(mut self, columns: Vec<String>) -> Self {
        self.columns = columns;
        self
    }

    pub fn values(mut self, values: Vec<Vec<Value>>) -> Self {
        self.values = values;
        self
    }

    pub fn on_conflict(mut self, on_conflict: OnConflict) -> Self {
        self.on_conflict = Some(on_conflict);
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UpdateStatement {
    pub table: TableRef,
    pub set: Vec<(String, Value)>,
    pub where_clause: Option<ConditionExpression>,
}

impl UpdateStatement {
    pub fn new(table: TableRef) -> Self {
        Self { table, set: Vec::new(), where_clause: None }
    }

    pub fn set(mut self, set: Vec<(String, Value)>) -> Self {
        self.set = set;
        self
    }

    pub fn where_clause(mut self, condition: ConditionExpression) -> Self {
        self.where_clause = Some(condition);
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeleteStatement {
    pub table: TableRef,
    pub where_clause: Option<ConditionExpression>,
}

impl DeleteStatement {
    pub fn new(table: TableRef) -> Self {
        Self { table, where_clause: None }
    }

    pub fn where_clause(mut self, condition: ConditionExpression) -> Self {
        self.where_clause = Some(condition);
        self
    }
}
