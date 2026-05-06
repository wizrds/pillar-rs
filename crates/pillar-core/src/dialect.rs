use std::fmt::{Debug, Formatter, Result as FmtResult};

use crate::{ast::Statement, errors::Error, value::Value};


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Feature {
    WindowFunctions,
    CommonTableExpressions,
    MaterializedViews,
    Partitioning,
    ArrayFunctions,
    JsonFunctions,
    ApproximateAggregates,
    NestedTypes,
}

#[derive(Debug, Clone)]
pub struct PreparedStatement {
    pub sql: String,
    pub params: Vec<Value>,
}

pub trait Dialect: Send + Sync {
    fn name(&self) -> &'static str;
    fn transpile(&self, statement: &Statement) -> Result<PreparedStatement, Error>;
    fn supports_feature(&self, feature: Feature) -> bool;
    fn quote_identifier(&self, identifier: &str) -> String;
    fn parameter_placeholder(&self, index: usize) -> String;
}

impl Debug for dyn Dialect {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "Dialect({})", self.name())
    }
}
