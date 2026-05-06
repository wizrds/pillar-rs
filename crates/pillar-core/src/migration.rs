use std::collections::{HashMap, HashSet};
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


struct RevisionGraph {
    children: HashMap<String, Vec<String>>,
    parents: HashMap<String, String>,
}

impl RevisionGraph {
    fn new(migrations: &[&MigrationRef]) -> Self {
        let mut children: HashMap<String, Vec<String>> = HashMap::new();
        let mut parents: HashMap<String, String> = HashMap::new();

        for migration in migrations {
            if let Some(prev) = migration.previous_id() {
                children
                    .entry(prev.to_string())
                    .or_default()
                    .push(migration.id().to_string());

                parents.insert(migration.id().to_string(), prev.to_string());
            }
        }

        Self { children, parents }
    }

    fn find_path<F>(&self, from: &str, to: &str, next: F) -> Option<Vec<String>>
    where
        F: Fn(&Self, &str) -> Vec<String>,
    {
        if from == to {
            return Some(vec![from.to_string()]);
        }

        let mut visited = HashSet::new();
        let mut queue = vec![(from.to_string(), vec![from.to_string()])];

        while let Some((cur, path)) = queue.pop() {
            if !visited.insert(cur.clone()) {
                continue;
            }

            for neighbor in next(self, &cur) {
                if neighbor == to {
                    return Some([path.clone(), vec![to.to_string()]].concat());
                }

                queue.push((neighbor.clone(), [path.clone(), vec![neighbor]].concat()));
            }
        }

        None
    }

    fn find_up_path(&self, from: &str, to: &str) -> Option<Vec<String>> {
        self.find_path(from, to, |graph, node| {
            graph.children
                .get(node)
                .cloned()
                .unwrap_or_default()
        })
    }

    fn find_down_path(&self, from: &str, to: &str) -> Option<Vec<String>> {
        self.find_path(from, to, |graph, node| {
            graph.parents
                .get(node)
                .map(|p| vec![p.clone()])
                .unwrap_or_default()
        })
    }
}


struct RevisionChain {
    revisions: HashMap<String, MigrationRef>,
    graph: RevisionGraph,
    head: Option<String>,
    tail: Option<String>,
}

impl RevisionChain {
    fn new(migrations: Vec<MigrationRef>) -> Self {
        let revisions: HashMap<String, MigrationRef> = migrations
            .into_iter()
            .map(|m| (m.id().to_string(), m))
            .collect();

        let graph = RevisionGraph::new(&revisions.values().collect::<Vec<_>>());

        let head = revisions
            .keys()
            .find(|id| !graph.children.contains_key(*id))
            .cloned();

        let tail = revisions
            .keys()
            .find(|id| !graph.parents.contains_key(*id))
            .cloned();

        Self { revisions, graph, head, tail }
    }

    fn get(&self, id: &str) -> Option<&MigrationRef> {
        self.revisions.get(id)
    }

    fn head(&self) -> Option<&str> {
        self.head.as_deref()
    }

    fn tail(&self) -> Option<&str> {
        self.tail.as_deref()
    }

    fn upgrade_path(&self, from: &str, to: &str) -> Option<Vec<&MigrationRef>> {
        self.graph
            .find_up_path(from, to)
            .map(|ids| ids.iter().filter_map(|id| self.get(id)).collect())
    }

    fn downgrade_path(&self, from: &str, to: &str) -> Option<Vec<&MigrationRef>> {
        self.graph
            .find_down_path(from, to)
            .map(|ids| ids.iter().filter_map(|id| self.get(id)).collect())
    }
}


pub struct MigrationRunner<M: Migrations> {
    chain: RevisionChain,
    _marker: std::marker::PhantomData<M>,
}

impl<M: Migrations> MigrationRunner<M> {
    pub fn new() -> Self {
        Self {
            chain: RevisionChain::new(M::migrations()),
            _marker: std::marker::PhantomData,
        }
    }

    pub async fn upgrade(&self, db: &dyn Database) -> Result<(), Error> {
        self.upgrade_to(
            db,
            self.chain
                .head()
                .ok_or_else(|| Error::invalid_query("no head revision found"))?,
        )
        .await
    }

    pub async fn downgrade(&self, db: &dyn Database) -> Result<(), Error> {
        self.downgrade_to(
            db,
            self.chain
                .tail()
                .ok_or_else(|| Error::invalid_query("no tail revision found"))?,
        )
        .await
    }

    pub async fn upgrade_to(&self, db: &dyn Database, target: &str) -> Result<(), Error> {
        self.apply(db, target, MigrationDirection::Up).await
    }

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


#[async_trait]
pub trait Migrator: AsDynDatabase + Send + Sync {
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

impl<D: Database> Migrator for D {}
