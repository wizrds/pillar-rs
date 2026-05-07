use crate::{
    column::IntoColumnRef,
    value::Value,
};


/// A single SQL condition predicate used in WHERE and HAVING clauses.
#[derive(Debug, Clone, PartialEq)]
pub enum ConditionExpression {
    /// Column equals value.
    Eq(String, Value),
    /// Column does not equal value.
    Ne(String, Value),
    /// Column is greater than value.
    Gt(String, Value),
    /// Column is greater than or equal to value.
    Gte(String, Value),
    /// Column is less than value.
    Lt(String, Value),
    /// Column is less than or equal to value.
    Lte(String, Value),
    /// Column value is in the given list.
    In(String, Vec<Value>),
    /// Column value is not in the given list.
    NotIn(String, Vec<Value>),
    /// Column is NULL.
    IsNull(String),
    /// Column is not NULL.
    IsNotNull(String),
    /// Column value matches the given SQL LIKE pattern.
    Like(String, String),
    /// Column value does not match the given SQL LIKE pattern.
    NotLike(String, String),
    /// Column value is between two values (inclusive).
    Between(String, Value, Value),
    /// Column value is not between two values.
    NotBetween(String, Value, Value),
    /// Both conditions must be true.
    And(Box<ConditionExpression>, Box<ConditionExpression>),
    /// Either condition must be true.
    Or(Box<ConditionExpression>, Box<ConditionExpression>),
    /// The condition must be false.
    Not(Box<ConditionExpression>),
}

impl ConditionExpression {
    /// Creates an equality condition.
    pub fn eq(column: impl IntoColumnRef, value: impl Into<Value>) -> Self {
        ConditionExpression::Eq(column.into_column_ref(), value.into())
    }

    /// Creates an inequality condition.
    pub fn ne(column: impl IntoColumnRef, value: impl Into<Value>) -> Self {
        ConditionExpression::Ne(column.into_column_ref(), value.into())
    }

    /// Creates a greater-than condition.
    pub fn gt(column: impl IntoColumnRef, value: impl Into<Value>) -> Self {
        ConditionExpression::Gt(column.into_column_ref(), value.into())
    }

    /// Creates a greater-than-or-equal condition.
    pub fn gte(column: impl IntoColumnRef, value: impl Into<Value>) -> Self {
        ConditionExpression::Gte(column.into_column_ref(), value.into())
    }

    /// Creates a less-than condition.
    pub fn lt(column: impl IntoColumnRef, value: impl Into<Value>) -> Self {
        ConditionExpression::Lt(column.into_column_ref(), value.into())
    }

    /// Creates a less-than-or-equal condition.
    pub fn lte(column: impl IntoColumnRef, value: impl Into<Value>) -> Self {
        ConditionExpression::Lte(column.into_column_ref(), value.into())
    }

    /// Creates an `IN` condition.
    pub fn in_list(column: impl IntoColumnRef, values: Vec<Value>) -> Self {
        ConditionExpression::In(column.into_column_ref(), values)
    }

    /// Creates a `NOT IN` condition.
    pub fn is_not_in(column: impl IntoColumnRef, values: Vec<Value>) -> Self {
        ConditionExpression::NotIn(column.into_column_ref(), values)
    }

    /// Creates an `IS NULL` condition.
    pub fn is_null(column: impl IntoColumnRef) -> Self {
        ConditionExpression::IsNull(column.into_column_ref())
    }

    /// Creates an `IS NOT NULL` condition.
    pub fn is_not_null(column: impl IntoColumnRef) -> Self {
        ConditionExpression::IsNotNull(column.into_column_ref())
    }

    /// Creates a `LIKE` condition.
    pub fn like(column: impl IntoColumnRef, pattern: impl Into<String>) -> Self {
        ConditionExpression::Like(column.into_column_ref(), pattern.into())
    }

    /// Creates a `NOT LIKE` condition.
    pub fn not_like(column: impl IntoColumnRef, pattern: impl Into<String>) -> Self {
        ConditionExpression::NotLike(column.into_column_ref(), pattern.into())
    }

    /// Creates a `BETWEEN` condition.
    pub fn between(column: impl IntoColumnRef, low: impl Into<Value>, high: impl Into<Value>) -> Self {
        ConditionExpression::Between(column.into_column_ref(), low.into(), high.into())
    }

    /// Creates a `NOT BETWEEN` condition.
    pub fn not_between(column: impl IntoColumnRef, low: impl Into<Value>, high: impl Into<Value>) -> Self {
        ConditionExpression::NotBetween(column.into_column_ref(), low.into(), high.into())
    }

    /// Combines this expression with another using `AND`.
    pub fn and(self, other: ConditionExpression) -> Self {
        ConditionExpression::And(Box::new(self), Box::new(other))
    }

    /// Combines this expression with another using `OR`.
    pub fn or(self, other: ConditionExpression) -> Self {
        ConditionExpression::Or(Box::new(self), Box::new(other))
    }

    /// Negates this expression with `NOT`.
    pub fn not(self) -> Self {
        ConditionExpression::Not(Box::new(self))
    }
}

/// A collection of [`ConditionExpression`](crate::condition::ConditionExpression) values combined with AND or OR.
#[derive(Debug, Clone, PartialEq)]
pub enum Condition {
    /// All contained expressions must be true (AND).
    All(Vec<ConditionExpression>),
    /// Any contained expression must be true (OR).
    Any(Vec<ConditionExpression>),
}

impl Condition {
    /// Creates an empty AND condition.
    pub fn all() -> Self {
        Condition::All(Vec::new())
    }

    /// Creates an empty OR condition.
    pub fn any() -> Self {
        Condition::Any(Vec::new())
    }

    /// Appends an expression to this condition.
    pub fn add(mut self, expr: ConditionExpression) -> Self {
        match &mut self {
            Condition::All(exprs) => exprs.push(expr),
            Condition::Any(exprs) => exprs.push(expr),
        }
        self
    }

    /// Appends an expression only when `condition` is true.
    pub fn add_if<F>(self, condition: bool, f: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        if condition {
            f(self)
        } else {
            self
        }
    }

    /// Appends an expression if `Some`, otherwise returns unchanged.
    pub fn add_option(self, expr: Option<ConditionExpression>) -> Self {
        match expr {
            Some(e) => self.add(e),
            None => self,
        }
    }

    /// Creates an AND condition from an iterator of expressions.
    pub fn all_of(iter: impl IntoIterator<Item = ConditionExpression>) -> Self {
        Condition::All(iter.into_iter().collect())
    }

    /// Creates an OR condition from an iterator of expressions.
    pub fn any_of(iter: impl IntoIterator<Item = ConditionExpression>) -> Self {
        Condition::Any(iter.into_iter().collect())
    }

    /// Returns `true` if this condition contains no expressions.
    pub fn is_empty(&self) -> bool {
        match self {
            Condition::All(exprs)
            | Condition::Any(exprs) => exprs.is_empty(),
        }
    }

    /// Folds all contained expressions into a single [`ConditionExpression`](crate::condition::ConditionExpression), or `None` if empty.
    pub fn to_expression(&self) -> Option<ConditionExpression> {
        match self {
            Condition::All(exprs) if exprs.is_empty() => None,
            Condition::All(exprs) if exprs.len() == 1 => Some(exprs[0].clone()),
            Condition::All(exprs) => {
                let mut iter = exprs.iter();
                let first = iter.next().unwrap().clone();
                Some(iter.fold(first, |acc, expr| acc.and(expr.clone())))
            },
            Condition::Any(exprs) if exprs.is_empty() => None,
            Condition::Any(exprs) if exprs.len() == 1 => Some(exprs[0].clone()),
            Condition::Any(exprs) => {
                let mut iter = exprs.iter();
                let first = iter.next().unwrap().clone();
                Some(iter.fold(first, |acc, expr| acc.or(expr.clone())))
            }
        }
    }
}

impl From<ConditionExpression> for Condition {
    fn from(expr: ConditionExpression) -> Self {
        Condition::All(vec![expr])
    }
}
