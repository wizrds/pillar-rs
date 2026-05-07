use std::fmt::{Debug, Formatter, Result as FmtResult};

use crate::{ast::Statement, errors::Error, value::Value};


/// An optional capability that a backend may or may not support.
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

/// A SQL string with its bound parameter values, ready to send to a backend.
#[derive(Debug, Clone)]
pub struct PreparedStatement {
    pub sql: String,
    pub params: Vec<Value>,
}

/// Translates pillar AST nodes into backend-specific SQL.
///
/// Each database backend provides its own implementation. Use
/// [`Database::dialect`](crate::database::Database::dialect) to retrieve the dialect
/// for a given connection.
pub trait Dialect: Send + Sync {
    /// Returns the name of this dialect (e.g. `"duckdb"` or `"clickhouse"`).
    fn name(&self) -> &'static str;

    /// Translates a [`Statement`](crate::ast::Statement) into a [`PreparedStatement`](crate::dialect::PreparedStatement).
    fn transpile(&self, statement: &Statement) -> Result<PreparedStatement, Error>;

    /// Returns `true` if this backend supports the given [`Feature`](crate::dialect::Feature).
    fn supports_feature(&self, feature: Feature) -> bool;

    /// Wraps an identifier in the backend-appropriate quoting characters.
    fn quote_identifier(&self, identifier: &str) -> String;

    /// Returns the parameter placeholder for the given zero-based index (e.g. `$1`, `?`).
    fn parameter_placeholder(&self, index: usize) -> String;
}

impl Debug for dyn Dialect {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "Dialect({})", self.name())
    }
}
