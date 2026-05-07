/// A reference to a table or view in a query, optionally with an alias.
#[derive(Debug, Clone, PartialEq)]
pub struct TableRef {
    pub name: String,
    pub alias: Option<String>,
}

impl TableRef {
    /// Creates a new [`TableRef`](crate::ast::TableRef) for the given table name.
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), alias: None }
    }

    /// Sets an alias for this table reference.
    pub fn alias(mut self, alias: impl Into<String>) -> Self {
        self.alias = Some(alias.into());
        self
    }
}

impl From<String> for TableRef {
    fn from(name: String) -> Self {
        Self::new(name)
    }
}

impl From<&str> for TableRef {
    fn from(name: &str) -> Self {
        Self::new(name)
    }
}
