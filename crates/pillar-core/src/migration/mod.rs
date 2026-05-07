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
use crate::{database::{AsDynDatabase, Database}, errors::Error};


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

    /// Applies all migrations up to the latest revision.
    pub async fn upgrade(&self, db: &dyn Database) -> Result<(), Error> {
        self.upgrade_to(
            db,
            self.chain
                .head()
                .ok_or_else(|| Error::invalid_query("no head revision found"))?,
        )
        .await
    }

    /// Reverts all migrations down to the earliest revision.
    pub async fn downgrade(&self, db: &dyn Database) -> Result<(), Error> {
        self.downgrade_to(
            db,
            self.chain
                .tail()
                .ok_or_else(|| Error::invalid_query("no tail revision found"))?,
        )
        .await
    }

    /// Applies migrations up to the given target revision ID.
    pub async fn upgrade_to(&self, db: &dyn Database, target: &str) -> Result<(), Error> {
        self.apply(db, target, MigrationDirection::Up).await
    }

    /// Reverts migrations down to the given target revision ID.
    pub async fn downgrade_to(&self, db: &dyn Database, target: &str) -> Result<(), Error> {
        self.apply(db, target, MigrationDirection::Down).await
    }

    async fn apply(
        &self,
        db: &dyn Database,
        target: &str,
        direction: MigrationDirection,
    ) -> Result<(), Error> {
        let op = MigrateOp::new(db);

        let path = match direction {
            MigrationDirection::Up => self
                .chain
                .upgrade_path(self.chain.tail().unwrap_or(""), target)
                .ok_or_else(|| {
                    Error::invalid_query(format!("no upgrade path to '{target}'"))
                })?,

            MigrationDirection::Down => self
                .chain
                .downgrade_path(self.chain.head().unwrap_or(""), target)
                .ok_or_else(|| {
                    Error::invalid_query(format!("no downgrade path to '{target}'"))
                })?,
        };

        for migration in path {
            match direction {
                MigrationDirection::Up => migration.up(&op).await?,
                MigrationDirection::Down => migration.down(&op).await?,
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
        MigrationRunner::<M>::new().upgrade(self.as_dyn()).await
    }

    async fn upgrade_to<M: Migrations + 'static>(&self, target: &str) -> Result<(), Error> {
        MigrationRunner::<M>::new().upgrade_to(self.as_dyn(), target).await
    }

    async fn downgrade<M: Migrations + 'static>(&self) -> Result<(), Error> {
        MigrationRunner::<M>::new().downgrade(self.as_dyn()).await
    }

    async fn downgrade_to<M: Migrations + 'static>(&self, target: &str) -> Result<(), Error> {
        MigrationRunner::<M>::new().downgrade_to(self.as_dyn(), target).await
    }
}
