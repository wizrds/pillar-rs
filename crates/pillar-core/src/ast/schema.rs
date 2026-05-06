use std::collections::HashMap;

use crate::{column::ColumnType, value::Value};

use super::{select::SelectStatement, ttl::TtlClause};


#[derive(Debug, Clone, PartialEq)]
pub struct ColumnDefinition {
    pub name: String,
    pub data_type: ColumnType,
    pub nullable: bool,
    pub primary_key: bool,
    pub default: Option<Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateTableStatement {
    pub name: String,
    pub columns: Vec<ColumnDefinition>,
    pub if_not_exists: bool,
    pub options: HashMap<String, String>,
    pub ttl: Option<TtlClause>,
}

impl CreateTableStatement {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            columns: Vec::new(),
            if_not_exists: false,
            options: HashMap::new(),
            ttl: None,
        }
    }

    pub fn columns(mut self, columns: Vec<ColumnDefinition>) -> Self {
        self.columns = columns;
        self
    }

    pub fn if_not_exists(mut self) -> Self {
        self.if_not_exists = true;
        self
    }

    pub fn option(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.options.insert(key.into(), value.into());
        self
    }

    pub fn ttl(mut self, ttl: TtlClause) -> Self {
        self.ttl = Some(ttl);
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlterTableStatement {
    pub name: String,
    pub add_columns: Vec<ColumnDefinition>,
    pub drop_columns: Vec<String>,
    pub ttl: Option<TtlClause>,
}

impl AlterTableStatement {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), add_columns: Vec::new(), drop_columns: Vec::new(), ttl: None }
    }

    pub fn add_columns(mut self, columns: Vec<ColumnDefinition>) -> Self {
        self.add_columns = columns;
        self
    }

    pub fn drop_columns(mut self, columns: Vec<String>) -> Self {
        self.drop_columns = columns;
        self
    }

    pub fn ttl(mut self, ttl: TtlClause) -> Self {
        self.ttl = Some(ttl);
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DropTableStatement {
    pub name: String,
    pub if_exists: bool,
}

impl DropTableStatement {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), if_exists: false }
    }

    pub fn if_exists(mut self) -> Self {
        self.if_exists = true;
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateViewStatement {
    pub name: String,
    pub query: SelectStatement,
    pub or_replace: bool,
    pub if_not_exists: bool,
    pub options: HashMap<String, String>,
}

impl CreateViewStatement {
    pub fn new(name: impl Into<String>, query: SelectStatement) -> Self {
        Self {
            name: name.into(),
            query,
            or_replace: false,
            if_not_exists: false,
            options: HashMap::new(),
        }
    }

    pub fn or_replace(mut self) -> Self {
        self.or_replace = true;
        self
    }

    pub fn if_not_exists(mut self) -> Self {
        self.if_not_exists = true;
        self
    }

    pub fn option(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.options.insert(key.into(), value.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateMaterializedViewStatement {
    pub name: String,
    pub query: SelectStatement,
    pub or_replace: bool,
    pub if_not_exists: bool,
    pub to_table: Option<String>,
    pub options: HashMap<String, String>,
}

impl CreateMaterializedViewStatement {
    pub fn new(name: impl Into<String>, query: SelectStatement) -> Self {
        Self {
            name: name.into(),
            query,
            or_replace: false,
            if_not_exists: false,
            to_table: None,
            options: HashMap::new(),
        }
    }

    pub fn or_replace(mut self) -> Self {
        self.or_replace = true;
        self
    }

    pub fn if_not_exists(mut self) -> Self {
        self.if_not_exists = true;
        self
    }

    pub fn to_table(mut self, table: impl Into<String>) -> Self {
        self.to_table = Some(table.into());
        self
    }

    pub fn option(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.options.insert(key.into(), value.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DropViewStatement {
    pub name: String,
    pub if_exists: bool,
    pub materialized: bool,
}

impl DropViewStatement {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), if_exists: false, materialized: false }
    }

    pub fn if_exists(mut self) -> Self {
        self.if_exists = true;
        self
    }

    pub fn materialized(mut self) -> Self {
        self.materialized = true;
        self
    }
}
