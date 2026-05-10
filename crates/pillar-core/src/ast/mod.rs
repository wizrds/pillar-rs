pub mod refs;
pub mod expression;
pub mod mutation;
pub mod schema;
pub mod select;
pub mod duration;
pub mod window;
pub mod compound;

pub use refs::*;
pub use expression::*;
pub use mutation::*;
pub use schema::*;
pub use select::*;
pub use duration::*;
pub use window::*;
pub use compound::*;

use crate::value::Value;


/// A SQL statement that can be executed against a database.
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    /// A `SELECT` query.
    Select(SelectStatement),
    /// An `INSERT` statement.
    Insert(InsertStatement),
    /// An `UPDATE` statement.
    Update(UpdateStatement),
    /// A `DELETE` statement.
    Delete(DeleteStatement),
    /// A `CREATE TABLE` statement.
    CreateTable(CreateTableStatement),
    /// An `ALTER TABLE` statement.
    AlterTable(AlterTableStatement),
    /// A `DROP TABLE` statement.
    DropTable(DropTableStatement),
    /// Checks whether a table with the given name exists.
    TableExists(String),
    /// A `CREATE VIEW` statement.
    CreateView(CreateViewStatement),
    /// A `CREATE MATERIALIZED VIEW` statement.
    CreateMaterializedView(CreateMaterializedViewStatement),
    /// A `DROP VIEW` statement.
    DropView(DropViewStatement),
    /// A compound set operation (`UNION`, `INTERSECT`, `EXCEPT`).
    Compound(CompoundSelect),
    /// A raw SQL string with positional bind parameters.
    Raw(String, Vec<Value>),
}

impl Statement {
    /// Creates a [`Statement::Select`](crate::ast::Statement::Select) from the given query.
    pub fn select(stmt: SelectStatement) -> Self {
        Self::Select(stmt)
    }

    /// Creates a [`Statement::Insert`](crate::ast::Statement::Insert) from the given statement.
    pub fn insert(stmt: InsertStatement) -> Self {
        Self::Insert(stmt)
    }

    /// Creates a [`Statement::Update`](crate::ast::Statement::Update) from the given statement.
    pub fn update(stmt: UpdateStatement) -> Self {
        Self::Update(stmt)
    }

    /// Creates a [`Statement::Delete`](crate::ast::Statement::Delete) from the given statement.
    pub fn delete(stmt: DeleteStatement) -> Self {
        Self::Delete(stmt)
    }

    /// Creates a [`Statement::CreateTable`](crate::ast::Statement::CreateTable) from the given statement.
    pub fn create_table(stmt: CreateTableStatement) -> Self {
        Self::CreateTable(stmt)
    }

    /// Creates a [`Statement::AlterTable`](crate::ast::Statement::AlterTable) from the given statement.
    pub fn alter_table(stmt: AlterTableStatement) -> Self {
        Self::AlterTable(stmt)
    }

    /// Creates a [`Statement::DropTable`](crate::ast::Statement::DropTable) from the given statement.
    pub fn drop_table(stmt: DropTableStatement) -> Self {
        Self::DropTable(stmt)
    }

    /// Creates a [`Statement::TableExists`](crate::ast::Statement::TableExists) check for the given table name.
    pub fn table_exists(name: impl Into<String>) -> Self {
        Self::TableExists(name.into())
    }

    /// Creates a [`Statement::CreateView`](crate::ast::Statement::CreateView) from the given statement.
    pub fn create_view(stmt: CreateViewStatement) -> Self {
        Self::CreateView(stmt)
    }

    /// Creates a [`Statement::CreateMaterializedView`](crate::ast::Statement::CreateMaterializedView) from the given statement.
    pub fn create_materialized_view(stmt: CreateMaterializedViewStatement) -> Self {
        Self::CreateMaterializedView(stmt)
    }

    /// Creates a [`Statement::DropView`](crate::ast::Statement::DropView) from the given statement.
    pub fn drop_view(stmt: DropViewStatement) -> Self {
        Self::DropView(stmt)
    }

    /// Creates a [`Statement::Compound`](crate::ast::Statement::Compound) from the given compound select.
    pub fn compound(stmt: CompoundSelect) -> Self {
        Self::Compound(stmt)
    }

    /// Creates a [`Statement::Raw`](crate::ast::Statement::Raw) from a SQL string with positional bind parameters.
    pub fn raw(
        sql: impl Into<String>,
        params: impl IntoIterator<Item = impl Into<Value>>,
    ) -> Self {
        Self::Raw(sql.into(), params.into_iter().map(Into::into).collect())
    }
}
