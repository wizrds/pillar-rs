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


/// Executes DDL statements against a database reference within a migration.
pub struct MigrateOp<'a> {
    database: &'a dyn Database,
}

impl<'a> MigrateOp<'a> {
    /// Creates a new [`MigrateOp`](crate::migration::MigrateOp) wrapping the given database.
    pub fn new(database: &'a dyn Database) -> Self {
        Self { database }
    }

    /// Executes a [`CreateTableStatement`](crate::ast::CreateTableStatement).
    pub async fn create_table(&self, stmt: CreateTableStatement) -> Result<ExecutionResult, Error> {
        self.database
            .execute(&Statement::CreateTable(stmt))
            .await
    }

    /// Executes an [`AlterTableStatement`](crate::ast::AlterTableStatement).
    pub async fn alter_table(&self, stmt: AlterTableStatement) -> Result<ExecutionResult, Error> {
        self.database
            .execute(&Statement::AlterTable(stmt))
            .await
    }

    /// Executes a [`DropTableStatement`](crate::ast::DropTableStatement).
    pub async fn drop_table(&self, stmt: DropTableStatement) -> Result<ExecutionResult, Error> {
        self.database
            .execute(&Statement::DropTable(stmt))
            .await
    }

    /// Executes a [`CreateViewStatement`](crate::ast::CreateViewStatement).
    pub async fn create_view(&self, stmt: CreateViewStatement) -> Result<ExecutionResult, Error> {
        self.database
            .execute(&Statement::CreateView(stmt))
            .await
    }

    /// Executes a [`CreateMaterializedViewStatement`](crate::ast::CreateMaterializedViewStatement).
    pub async fn create_materialized_view(&self, stmt: CreateMaterializedViewStatement) -> Result<ExecutionResult, Error> {
        self.database
            .execute(&Statement::CreateMaterializedView(stmt))
            .await
    }

    /// Executes a [`DropViewStatement`](crate::ast::DropViewStatement).
    pub async fn drop_view(&self, stmt: DropViewStatement) -> Result<ExecutionResult, Error> {
        self.database
            .execute(&Statement::DropView(stmt))
            .await
    }

    /// Executes an arbitrary [`Statement`](crate::ast::Statement).
    pub async fn execute(&self, statement: &Statement) -> Result<ExecutionResult, Error> {
        self.database
            .execute(statement)
            .await
    }
}


/// A single database migration with upgrade and downgrade operations.
#[async_trait]
pub trait Migration: Send + Sync {
    /// Returns the unique identifier for this migration.
    fn id(&self) -> &'static str;
    /// Returns the identifier of the migration that precedes this one, if any.
    fn previous_id(&self) -> Option<&'static str>;

    /// Applies the migration.
    async fn up(&self, op: &MigrateOp<'_>) -> Result<(), Error>;
    /// Reverts the migration.
    async fn down(&self, op: &MigrateOp<'_>) -> Result<(), Error>;
}

/// A boxed [`Migration`](crate::migration::Migration) trait object.
pub type MigrationRef = Box<dyn Migration>;

/// A collection of [`MigrationRef`](crate::migration::MigrationRef) values for a schema.
pub trait Migrations: Send + Sync {
    /// Returns all migrations in this collection.
    fn migrations() -> Vec<MigrationRef>;
}

/// The direction of a migration run.
pub enum MigrationDirection {
    /// Apply migrations forward.
    Up,
    /// Revert migrations.
    Down,
}


/// Provides upgrade and downgrade operations on a database.
#[async_trait]
pub trait Migrator: AsDynDatabase + Send + Sync {
    /// Applies all migrations to the latest revision.
    async fn upgrade<M: Migrations + 'static>(&self) -> Result<(), Error>;
    /// Applies migrations up to the given target revision.
    async fn upgrade_to<M: Migrations + 'static>(&self, target: &str) -> Result<(), Error>;
    /// Reverts all migrations to the earliest revision.
    async fn downgrade<M: Migrations + 'static>(&self) -> Result<(), Error>;
    /// Reverts migrations down to the given target revision.
    async fn downgrade_to<M: Migrations + 'static>(&self, target: &str) -> Result<(), Error>;
}
