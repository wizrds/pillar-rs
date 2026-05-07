/// The result of a statement that does not return rows.
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// Number of rows affected by the statement.
    pub rows_affected: usize,
    pub metadata: Option<serde_json::Value>,
}
