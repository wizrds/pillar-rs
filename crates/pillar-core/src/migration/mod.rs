pub mod graph;
pub mod traits;

pub use graph::{RevisionGraph, RevisionChain};
pub use traits::{
    MigrateOp,
    Migration,
    MigrationDirection,
    MigrationRef,
    Migrations,
    Migrator,
};

use async_trait::async_trait;
use crate::{
    ast::{
        ColumnDefinition, ColumnType, CreateTableStatement, InsertStatement,
        OnConflict, OnConflictAction, Projection, SelectStatement, Statement, TableRef,
    },
    database::{AsDynDatabase, Database},
    errors::Error,
    value::Value,
};


/// Runs migrations for a given [`Migrations`](crate::migration::Migrations) collection against a database.
pub struct MigrationRunner<M: Migrations> {
    chain: RevisionChain,
    _marker: std::marker::PhantomData<M>,
}

impl<M: Migrations> MigrationRunner<M> {
    /// Creates a new [`MigrationRunner`](crate::migration::MigrationRunner) for the given migration collection.
    pub fn new() -> Self {
        Self {
            chain: RevisionChain::new(M::migrations()),
            _marker: std::marker::PhantomData,
        }
    }

    async fn get_revision_id(&self, database: &dyn Database) -> Result<Option<String>, Error> {
        let result = database
            .query(&Statement::TableExists("_migrations".to_owned()))
            .await?;

        let count = result
            .get_as::<u64>(0, 0)
            .unwrap_or_else(|| {
                result
                    .get_as::<i64>(0, 0)
                    .unwrap_or(0) as u64
            });

        if count == 0 {
            return Ok(None);
        }

        Ok(
            database
                .query(&Statement::Select(
                    SelectStatement::new(TableRef::new("_migrations"))
                        .projections(vec![Projection::Column("revision_id".into())])
                        .limit(1u64),
                ))
                .await?
                .get_as::<String>(0, 0)
        )
    }

    async fn set_revision_id(&self, database: &dyn Database, revision_id: &str) -> Result<(), Error> {
        database.execute(&Statement::CreateTable(
            CreateTableStatement::new("_migrations")
                .if_not_exists()
                .columns(vec![
                    ColumnDefinition::new("revision_id", ColumnType::String).primary_key(),
                ]),
        ))
        .await?;

        database.execute(&Statement::Insert(
            InsertStatement::new(TableRef::new("_migrations"))
                .columns(["revision_id"])
                .values(vec![vec![Value::String(revision_id.to_owned())]])
                .on_conflict(OnConflict {
                    target: vec!["revision_id".into()],
                    action: OnConflictAction::DoUpdate {
                        set: vec![("revision_id".into(), Value::String(revision_id.to_owned()))],
                        where_clause: None,
                    },
                }),
        ))
        .await?;

        Ok(())
    }

    /// Applies all migrations up to the latest revision.
    pub async fn upgrade(&self, database: &dyn Database) -> Result<(), Error> {
        self.upgrade_to(
            database,
            self.chain
                .head()
                .ok_or_else(|| Error::invalid_query("no head revision found"))?,
        )
        .await
    }

    /// Reverts all migrations down to the earliest revision.
    pub async fn downgrade(&self, database: &dyn Database) -> Result<(), Error> {
        self.downgrade_to(
            database,
            self.chain
                .tail()
                .ok_or_else(|| Error::invalid_query("no tail revision found"))?,
        )
        .await
    }

    /// Applies migrations up to the given target revision ID.
    pub async fn upgrade_to(&self, database: &dyn Database, target: &str) -> Result<(), Error> {
        self.apply(database, target, MigrationDirection::Up).await
    }

    /// Reverts migrations down to the given target revision ID.
    pub async fn downgrade_to(&self, database: &dyn Database, target: &str) -> Result<(), Error> {
        self.apply(database, target, MigrationDirection::Down).await
    }

    async fn apply(
        &self,
        database: &dyn Database,
        target: &str,
        direction: MigrationDirection,
    ) -> Result<(), Error> {
        let current_revision = self.get_revision_id(database).await?;
        let op = MigrateOp::new(database);

        let path = match direction {
            MigrationDirection::Up => {
                let from = current_revision
                    .as_deref()
                    .unwrap_or_else(|| self.chain.tail().unwrap_or(""));

                let path = self.chain
                    .upgrade_path(from, target)
                    .ok_or_else(|| {
                        Error::invalid_query(format!("no upgrade path to '{target}'"))
                    })?;

                if current_revision.is_some() {
                    path.into_iter().skip(1).collect()
                } else {
                    path
                }
            },
            MigrationDirection::Down => {
                let from = current_revision
                    .as_deref()
                    .unwrap_or_else(|| self.chain.head().unwrap_or(""));

                self.chain
                    .downgrade_path(from, target)
                    .ok_or_else(|| {
                        Error::invalid_query(format!("no downgrade path to '{target}'"))
                    })?
            },
        };

        for migration in path {
            match direction {
                MigrationDirection::Up => {
                    migration.up(&op).await?;
                    self.set_revision_id(database, migration.id()).await?;
                }

                MigrationDirection::Down => {
                    migration.down(&op).await?;
                    self.set_revision_id(database, migration.previous_id().unwrap_or("")).await?;
                }
            }
        }

        Ok(())
    }
}

impl<M: Migrations> Default for MigrationRunner<M> {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl<D: Database> Migrator for D {
    async fn upgrade<M: Migrations + 'static>(&self) -> Result<(), Error> {
        MigrationRunner::<M>::new()
            .upgrade(self.as_dyn())
            .await
    }

    async fn upgrade_to<M: Migrations + 'static>(&self, target: &str) -> Result<(), Error> {
        MigrationRunner::<M>::new()
            .upgrade_to(self.as_dyn(), target)
            .await
    }

    async fn downgrade<M: Migrations + 'static>(&self) -> Result<(), Error> {
        MigrationRunner::<M>::new()
            .downgrade(self.as_dyn())
            .await
    }

    async fn downgrade_to<M: Migrations + 'static>(&self, target: &str) -> Result<(), Error> {
        MigrationRunner::<M>::new()
            .downgrade_to(self.as_dyn(), target)
            .await
    }
}
