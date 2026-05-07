use std::collections::{HashMap, HashSet};

use crate::migration::traits::MigrationRef;


/// A directed graph of migration revision relationships.
pub struct RevisionGraph {
    /// Maps each revision ID to the IDs of its child revisions.
    pub children: HashMap<String, Vec<String>>,
    /// Maps each revision ID to its parent revision ID.
    pub parents: HashMap<String, String>,
}

impl RevisionGraph {
    /// Builds a [`RevisionGraph`](crate::migration::RevisionGraph) from a slice of migration references.
    pub fn new(migrations: &[&MigrationRef]) -> Self {
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

    /// Returns the ordered revision IDs along the upgrade path from `from` to `to`, or `None` if no path exists.
    pub fn find_up_path(&self, from: &str, to: &str) -> Option<Vec<String>> {
        self.find_path(from, to, |graph, node| {
            graph.children
                .get(node)
                .cloned()
                .unwrap_or_default()
        })
    }

    /// Returns the ordered revision IDs along the downgrade path from `from` to `to`, or `None` if no path exists.
    pub fn find_down_path(&self, from: &str, to: &str) -> Option<Vec<String>> {
        self.find_path(from, to, |graph, node| {
            graph.parents
                .get(node)
                .map(|p| vec![p.clone()])
                .unwrap_or_default()
        })
    }
}


/// An ordered chain of migrations with graph-based path resolution.
pub struct RevisionChain {
    revisions: HashMap<String, MigrationRef>,
    graph: RevisionGraph,
    head: Option<String>,
    tail: Option<String>,
}

impl RevisionChain {
    /// Builds a [`RevisionChain`](crate::migration::RevisionChain) from a list of migration references.
    pub fn new(migrations: Vec<MigrationRef>) -> Self {
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

    /// Returns the migration with the given ID, if it exists.
    pub fn get(&self, id: &str) -> Option<&MigrationRef> {
        self.revisions.get(id)
    }

    /// Returns the ID of the latest migration in the chain.
    pub fn head(&self) -> Option<&str> {
        self.head.as_deref()
    }

    /// Returns the ID of the earliest migration in the chain.
    pub fn tail(&self) -> Option<&str> {
        self.tail.as_deref()
    }

    /// Returns the ordered migrations along the upgrade path from `from` to `to`.
    pub fn upgrade_path(&self, from: &str, to: &str) -> Option<Vec<&MigrationRef>> {
        self.graph
            .find_up_path(from, to)
            .map(|ids| ids.iter().filter_map(|id| self.get(id)).collect())
    }

    /// Returns the ordered migrations along the downgrade path from `from` to `to`.
    pub fn downgrade_path(&self, from: &str, to: &str) -> Option<Vec<&MigrationRef>> {
        self.graph
            .find_down_path(from, to)
            .map(|ids| ids.iter().filter_map(|id| self.get(id)).collect())
    }
}
