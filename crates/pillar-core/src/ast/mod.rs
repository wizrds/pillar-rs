pub mod table;
pub mod mutation;
pub mod schema;
pub mod select;
pub mod ttl;

pub use table::*;
pub use mutation::*;
pub use schema::*;
pub use select::*;
pub use ttl::*;

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
    /// A raw SQL string with positional bind parameters.
    Raw(String, Vec<Value>),
}
