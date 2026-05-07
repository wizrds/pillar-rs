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


#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Select(SelectStatement),
    Insert(InsertStatement),
    Update(UpdateStatement),
    Delete(DeleteStatement),
    CreateTable(CreateTableStatement),
    AlterTable(AlterTableStatement),
    DropTable(DropTableStatement),
    CreateView(CreateViewStatement),
    CreateMaterializedView(CreateMaterializedViewStatement),
    DropView(DropViewStatement),
    Raw(String, Vec<Value>),
}
