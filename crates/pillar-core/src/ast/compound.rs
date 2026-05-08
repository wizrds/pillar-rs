use crate::ast::select::SelectStatement;


/// A set operation combining two or more `SELECT` statements.
#[derive(Debug, Clone, PartialEq)]
pub struct CompoundSelect {
    pub operator: SetOperator,
    pub left: Box<SelectStatement>,
    pub right: Box<SelectStatement>,
}

impl CompoundSelect {
    /// Creates a new compound select statement with the given operator.
    pub fn new(
        operator: SetOperator,
        left: SelectStatement,
        right: SelectStatement,
    ) -> Self {
        Self {
            operator,
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    /// Creates a `UNION ALL` of two queries.
    pub fn union_all(left: SelectStatement, right: SelectStatement) -> Self {
        Self::new(SetOperator::UnionAll, left, right)
    }

    /// Creates a `UNION` (distinct) of two queries.
    pub fn union(left: SelectStatement, right: SelectStatement) -> Self {
        Self::new(SetOperator::Union, left, right)
    }

    /// Creates an `INTERSECT` of two queries.
    pub fn intersect(left: SelectStatement, right: SelectStatement) -> Self {
        Self::new(SetOperator::Intersect, left, right)
    }

    /// Creates an `EXCEPT` of two queries.
    pub fn except(left: SelectStatement, right: SelectStatement) -> Self {
        Self::new(SetOperator::Except, left, right)
    }
}

/// The set operator joining two queries in a [`CompoundSelect`](crate::ast::CompoundSelect).
#[derive(Debug, Clone, PartialEq)]
pub enum SetOperator {
    /// `UNION ALL` — keeps duplicate rows.
    UnionAll,
    /// `UNION` — removes duplicate rows.
    Union,
    /// `INTERSECT` — rows present in both queries.
    Intersect,
    /// `EXCEPT` — rows in the left query not in the right.
    Except,
}
