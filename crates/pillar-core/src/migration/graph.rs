use std::collections::{HashMap, HashSet};

use crate::migration::traits::MigrationRef;


pub struct RevisionGraph {
    pub children: HashMap<String, Vec<String>>,
    pub parents: HashMap<String, String>,
}

impl RevisionGraph {
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

    pub fn find_up_path(&self, from: &str, to: &str) -> Option<Vec<String>> {
        self.find_path(from, to, |graph, node| {
            graph.children
                .get(node)
                .cloned()
                .unwrap_or_default()
        })
    }

    pub fn find_down_path(&self, from: &str, to: &str) -> Option<Vec<String>> {
        self.find_path(from, to, |graph, node| {
            graph.parents
                .get(node)
                .map(|p| vec![p.clone()])
                .unwrap_or_default()
        })
    }
}


pub struct RevisionChain {
    revisions: HashMap<String, MigrationRef>,
    graph: RevisionGraph,
    head: Option<String>,
    tail: Option<String>,
}

impl RevisionChain {
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

    pub fn get(&self, id: &str) -> Option<&MigrationRef> {
        self.revisions.get(id)
    }

    pub fn head(&self) -> Option<&str> {
        self.head.as_deref()
    }

    pub fn tail(&self) -> Option<&str> {
        self.tail.as_deref()
    }

    pub fn upgrade_path(&self, from: &str, to: &str) -> Option<Vec<&MigrationRef>> {
        self.graph
            .find_up_path(from, to)
            .map(|ids| ids.iter().filter_map(|id| self.get(id)).collect())
    }

    pub fn downgrade_path(&self, from: &str, to: &str) -> Option<Vec<&MigrationRef>> {
        self.graph
            .find_down_path(from, to)
            .map(|ids| ids.iter().filter_map(|id| self.get(id)).collect())
    }
}
