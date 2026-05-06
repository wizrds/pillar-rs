pub mod mutation;
pub mod schema;
pub mod select;
pub mod ttl;

pub use mutation::{
    DeleteStatement, InsertStatement, OnConflict, OnConflictAction, UpdateStatement,
};
pub use schema::{
    AlterTableStatement, ColumnDefinition, CreateMaterializedViewStatement, CreateTableStatement,
    CreateViewStatement, DropTableStatement, DropViewStatement,
};
pub use select::{
    AggregateFunction, BinaryOperator, CountArg, Expression, Join, JoinType, NullsOrder, OrderBy,
    OrderDirection, Projection, SelectStatement, TableRef,
};
pub use ttl::{Interval, IntervalUnit, TtlAction, TtlClause};

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
