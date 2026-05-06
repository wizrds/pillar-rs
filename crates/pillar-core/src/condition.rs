use crate::value::Value;


#[derive(Debug, Clone, PartialEq)]
pub enum ConditionExpression {
    Eq(String, Value),
    Ne(String, Value),
    Gt(String, Value),
    Gte(String, Value),
    Lt(String, Value),
    Lte(String, Value),
    In(String, Vec<Value>),
    NotIn(String, Vec<Value>),
    IsNull(String),
    IsNotNull(String),
    Like(String, String),
    NotLike(String, String),
    Between(String, Value, Value),
    NotBetween(String, Value, Value),
    And(Box<ConditionExpression>, Box<ConditionExpression>),
    Or(Box<ConditionExpression>, Box<ConditionExpression>),
    Not(Box<ConditionExpression>),
}

impl ConditionExpression {
    pub fn eq(column: impl Into<String>, value: impl Into<Value>) -> Self {
        ConditionExpression::Eq(column.into(), value.into())
    }

    pub fn ne(column: impl Into<String>, value: impl Into<Value>) -> Self {
        ConditionExpression::Ne(column.into(), value.into())
    }

    pub fn gt(column: impl Into<String>, value: impl Into<Value>) -> Self {
        ConditionExpression::Gt(column.into(), value.into())
    }

    pub fn gte(column: impl Into<String>, value: impl Into<Value>) -> Self {
        ConditionExpression::Gte(column.into(), value.into())
    }

    pub fn lt(column: impl Into<String>, value: impl Into<Value>) -> Self {
        ConditionExpression::Lt(column.into(), value.into())
    }

    pub fn lte(column: impl Into<String>, value: impl Into<Value>) -> Self {
        ConditionExpression::Lte(column.into(), value.into())
    }

    pub fn in_list(column: impl Into<String>, values: Vec<Value>) -> Self {
        ConditionExpression::In(column.into(), values)
    }

    pub fn is_not_in(column: impl Into<String>, values: Vec<Value>) -> Self {
        ConditionExpression::NotIn(column.into(), values)
    }

    pub fn is_null(column: impl Into<String>) -> Self {
        ConditionExpression::IsNull(column.into())
    }

    pub fn is_not_null(column: impl Into<String>) -> Self {
        ConditionExpression::IsNotNull(column.into())
    }

    pub fn like(column: impl Into<String>, pattern: impl Into<String>) -> Self {
        ConditionExpression::Like(column.into(), pattern.into())
    }

    pub fn not_like(column: impl Into<String>, pattern: impl Into<String>) -> Self {
        ConditionExpression::NotLike(column.into(), pattern.into())
    }

    pub fn between(column: impl Into<String>, low: impl Into<Value>, high: impl Into<Value>) -> Self {
        ConditionExpression::Between(column.into(), low.into(), high.into())
    }

    pub fn not_between(column: impl Into<String>, low: impl Into<Value>, high: impl Into<Value>) -> Self {
        ConditionExpression::NotBetween(column.into(), low.into(), high.into())
    }

    pub fn and(self, other: ConditionExpression) -> Self {
        ConditionExpression::And(Box::new(self), Box::new(other))
    }

    pub fn or(self, other: ConditionExpression) -> Self {
        ConditionExpression::Or(Box::new(self), Box::new(other))
    }

    pub fn not(self) -> Self {
        ConditionExpression::Not(Box::new(self))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Condition {
    All(Vec<ConditionExpression>),
    Any(Vec<ConditionExpression>),
}

impl Condition {
    pub fn all() -> Self {
        Condition::All(Vec::new())
    }

    pub fn any() -> Self {
        Condition::Any(Vec::new())
    }

    pub fn add(mut self, expr: ConditionExpression) -> Self {
        match &mut self {
            Condition::All(exprs) => exprs.push(expr),
            Condition::Any(exprs) => exprs.push(expr),
        }
        self
    }

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

    pub fn add_option(self, expr: Option<ConditionExpression>) -> Self {
        match expr {
            Some(e) => self.add(e),
            None => self,
        }
    }

    pub fn all_of<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = ConditionExpression>,
    {
        Condition::All(iter.into_iter().collect())
    }

    pub fn any_of<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = ConditionExpression>,
    {
        Condition::Any(iter.into_iter().collect())
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Condition::All(exprs)
            | Condition::Any(exprs) => exprs.is_empty(),
        }
    }

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
