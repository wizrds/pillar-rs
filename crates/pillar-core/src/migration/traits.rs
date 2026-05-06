use async_trait::async_trait;

use crate::{
    ast::{
        AlterTableStatement,
        CreateMaterializedViewStatement,
        CreateTableStatement,
        CreateViewStatement,
        DropTableStatement,
        DropViewStatement,
        Statement,
    },
    errors::Error,
    database::{AsDynDatabase, Database, ExecutionResult},
};


pub struct MigrateOp<'a> {
    database: &'a dyn Database,
}

impl<'a> MigrateOp<'a> {
    pub fn new(database: &'a dyn Database) -> Self {
        Self { database }
    }

    pub async fn create_table(&self, stmt: CreateTableStatement) -> Result<ExecutionResult, Error> {
        self.database
            .execute(&Statement::CreateTable(stmt))
            .await
    }

    pub async fn alter_table(&self, stmt: AlterTableStatement) -> Result<ExecutionResult, Error> {
        self.database
            .execute(&Statement::AlterTable(stmt))
            .await
    }

    pub async fn drop_table(&self, stmt: DropTableStatement) -> Result<ExecutionResult, Error> {
        self.database
            .execute(&Statement::DropTable(stmt))
            .await
    }

    pub async fn create_view(&self, stmt: CreateViewStatement) -> Result<ExecutionResult, Error> {
        self.database
            .execute(&Statement::CreateView(stmt))
            .await
    }

    pub async fn create_materialized_view(&self, stmt: CreateMaterializedViewStatement) -> Result<ExecutionResult, Error> {
        self.database
            .execute(&Statement::CreateMaterializedView(stmt))
            .await
    }

    pub async fn drop_view(&self, stmt: DropViewStatement) -> Result<ExecutionResult, Error> {
        self.database
            .execute(&Statement::DropView(stmt))
            .await
    }

    pub async fn execute(&self, statement: &Statement) -> Result<ExecutionResult, Error> {
        self.database
            .execute(statement)
            .await
    }
}


#[async_trait]
pub trait Migration: Send + Sync {
    fn id(&self) -> &'static str;
    fn previous_id(&self) -> Option<&'static str>;

    async fn up(&self, op: &MigrateOp<'_>) -> Result<(), Error>;
    async fn down(&self, op: &MigrateOp<'_>) -> Result<(), Error>;
}

pub type MigrationRef = Box<dyn Migration>;

pub trait Migrations: Send + Sync {
    fn migrations() -> Vec<MigrationRef>;
}

pub enum MigrationDirection {
    Up,
    Down,
}


#[async_trait]
pub trait Migrator: AsDynDatabase + Send + Sync {
    async fn upgrade<M: Migrations + 'static>(&self) -> Result<(), Error>;
    async fn upgrade_to<M: Migrations + 'static>(&self, target: &str) -> Result<(), Error>;
    async fn downgrade<M: Migrations + 'static>(&self) -> Result<(), Error>;
    async fn downgrade_to<M: Migrations + 'static>(&self, target: &str) -> Result<(), Error>;
}
