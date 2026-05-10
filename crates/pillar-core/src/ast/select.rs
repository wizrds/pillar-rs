use crate::{
    condition::ConditionExpression,
    ast::{
        refs::{ColumnRef, TableRef},
        expression::{AggregateFunction, Expression},
    },
};


/// A single CTE (Common Table Expression) in a `WITH` clause.
#[derive(Debug, Clone, PartialEq)]
pub struct Cte {
    pub name: String,
    pub columns: Vec<ColumnRef>,
    pub query: Box<SelectStatement>,
    pub recursive: bool,
}

impl Cte {
    /// Creates a non-recursive CTE with the given name and query.
    pub fn new(name: impl Into<String>, query: SelectStatement) -> Self {
        Self {
            name: name.into(),
            columns: Vec::new(),
            query: Box::new(query),
            recursive: false,
        }
    }

    /// Creates a recursive CTE with the given name and query.
    pub fn recursive(name: impl Into<String>, query: SelectStatement) -> Self {
        Self {
            name: name.into(),
            columns: Vec::new(),
            query: Box::new(query),
            recursive: true,
        }
    }

    /// Specifies the optional column list for this CTE.
    pub fn columns(
        mut self,
        cols: impl IntoIterator<Item = impl Into<ColumnRef>>,
    ) -> Self {
        self.columns = cols.into_iter().map(Into::into).collect();
        self
    }
}


/// A column projection in a [`SelectStatement`](crate::ast::SelectStatement).
#[derive(Debug, Clone, PartialEq)]
pub enum Projection {
    /// Selects all columns (`*`).
    All,
    /// Selects a single named column.
    Column(ColumnRef),
    /// Selects a column with an alias.
    ColumnAlias(ColumnRef, String),
    /// Selects an aggregate expression.
    Aggregate(AggregateFunction),
    /// Selects an arbitrary expression.
    Expression(Expression),
    /// Wraps any projection with an output alias.
    Aliased(Box<Projection>, String),
}

impl Projection {
    /// Selects all columns (`*`).
    pub fn all() -> Self {
        Self::All
    }

    /// Selects a single named column.
    pub fn column(col: impl Into<ColumnRef>) -> Self {
        Self::Column(col.into())
    }

    /// Selects a column with an output alias.
    pub fn column_alias(col: impl Into<ColumnRef>, alias: impl Into<String>) -> Self {
        Self::ColumnAlias(col.into(), alias.into())
    }

    /// Selects an aggregate function.
    pub fn aggregate(f: AggregateFunction) -> Self {
        Self::Aggregate(f)
    }

    /// Selects an arbitrary expression.
    pub fn expr(e: impl Into<Expression>) -> Self {
        Self::Expression(e.into())
    }

    /// Wraps any projection with an output alias.
    pub fn aliased(inner: impl Into<Projection>, alias: impl Into<String>) -> Self {
        Self::Aliased(Box::new(inner.into()), alias.into())
    }

    /// Attaches an output alias to this projection.
    pub fn alias(self, alias: impl Into<String>) -> Self {
        Self::Aliased(Box::new(self), alias.into())
    }
}

/// A join clause in a [`SelectStatement`](crate::ast::SelectStatement).
#[derive(Debug, Clone, PartialEq)]
pub struct Join {
    pub table: TableRef,
    pub on: ConditionExpression,
    pub join_type: JoinType,
}

impl Join {
    /// Creates a new [`Join`](crate::ast::Join) with the given type, table, and ON condition.
    pub fn new(join_type: JoinType, table: impl Into<TableRef>, on: ConditionExpression) -> Self {
        Self { join_type, table: table.into(), on }
    }

    /// Creates an `INNER JOIN`.
    pub fn inner(table: impl Into<TableRef>, on: ConditionExpression) -> Self {
        Self::new(JoinType::Inner, table, on)
    }

    /// Creates a `LEFT JOIN`.
    pub fn left(table: impl Into<TableRef>, on: ConditionExpression) -> Self {
        Self::new(JoinType::Left, table, on)
    }

    /// Creates a `RIGHT JOIN`.
    pub fn right(table: impl Into<TableRef>, on: ConditionExpression) -> Self {
        Self::new(JoinType::Right, table, on)
    }

    /// Creates a `FULL OUTER JOIN`.
    pub fn full(table: impl Into<TableRef>, on: ConditionExpression) -> Self {
        Self::new(JoinType::Full, table, on)
    }

    /// Creates a `CROSS JOIN`.
    pub fn cross(table: impl Into<TableRef>, on: ConditionExpression) -> Self {
        Self::new(JoinType::Cross, table, on)
    }
}

/// The type of join to perform.
#[derive(Debug, Clone, PartialEq)]
pub enum JoinType {
    /// `INNER JOIN`.
    Inner,
    /// `LEFT JOIN`.
    Left,
    /// `RIGHT JOIN`.
    Right,
    /// `FULL OUTER JOIN`.
    Full,
    /// `CROSS JOIN`.
    Cross,
}

/// A single column ordering directive in a [`SelectStatement`](crate::ast::SelectStatement).
#[derive(Debug, Clone, PartialEq)]
pub struct OrderBy {
    pub column: ColumnRef,
    pub direction: OrderDirection,
    pub nulls: Option<NullsOrder>,
}

impl OrderBy {
    /// Orders by the given column ascending.
    pub fn asc(column: impl Into<ColumnRef>) -> Self {
        Self { column: column.into(), direction: OrderDirection::Asc, nulls: None }
    }

    /// Orders by the given column descending.
    pub fn desc(column: impl Into<ColumnRef>) -> Self {
        Self { column: column.into(), direction: OrderDirection::Desc, nulls: None }
    }

    /// Sets the NULL ordering for this directive.
    pub fn nulls(mut self, nulls: NullsOrder) -> Self {
        self.nulls = Some(nulls);
        self
    }
}

/// The sort direction for an [`OrderBy`](crate::ast::OrderBy).
#[derive(Debug, Clone, PartialEq)]
pub enum OrderDirection {
    /// Ascending order (`ASC`).
    Asc,
    /// Descending order (`DESC`).
    Desc,
}

/// Whether nulls appear first or last in an [`OrderBy`](crate::ast::OrderBy).
#[derive(Debug, Clone, PartialEq)]
pub enum NullsOrder {
    /// Nulls sort before non-null values (`NULLS FIRST`).
    First,
    /// Nulls sort after non-null values (`NULLS LAST`).
    Last,
}

/// The source of a `FROM` clause: either a table reference or a subquery.
#[derive(Debug, Clone, PartialEq)]
pub enum FromSource {
    /// A concrete table or view reference.
    Table(TableRef),
    /// A derived table (subquery) with a required alias.
    Subquery {
        query: Box<SelectStatement>,
        alias: String,
    },
}

impl FromSource {
    /// Creates a `FromSource` from a table reference.
    pub fn table(table: impl Into<TableRef>) -> Self {
        Self::Table(table.into())
    }

    /// Creates a `FromSource` from a subquery with a required alias.
    pub fn subquery(query: SelectStatement, alias: impl Into<String>) -> Self {
        Self::Subquery { query: Box::new(query), alias: alias.into() }
    }
}

impl From<TableRef> for FromSource {
    fn from(t: TableRef) -> Self {
        Self::Table(t)
    }
}

impl From<&str> for FromSource {
    fn from(s: &str) -> Self {
        Self::Table(TableRef::from(s))
    }
}

impl From<String> for FromSource {
    fn from(s: String) -> Self {
        Self::Table(TableRef::from(s))
    }
}

/// AST node for a `SELECT` statement.
#[derive(Debug, Clone, PartialEq)]
pub struct SelectStatement {
    pub distinct: bool,
    pub projections: Vec<Projection>,
    pub from: FromSource,
    pub joins: Vec<Join>,
    pub where_clause: Option<ConditionExpression>,
    pub group_by: Vec<ColumnRef>,
    pub having: Option<ConditionExpression>,
    pub order_by: Vec<OrderBy>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
    pub with: Vec<Cte>,
}

impl SelectStatement {
    /// Creates a new [`SelectStatement`](crate::ast::SelectStatement) selecting all columns from the given table.
    pub fn new(from: impl Into<FromSource>) -> Self {
        Self {
            distinct: false,
            projections: vec![Projection::All],
            from: from.into(),
            joins: Vec::new(),
            where_clause: None,
            group_by: Vec::new(),
            having: None,
            order_by: Vec::new(),
            limit: None,
            offset: None,
            with: Vec::new(),
        }
    }

    /// Adds a CTE to the `WITH` clause.
    pub fn with(mut self, cte: Cte) -> Self {
        self.with.push(cte);
        self
    }

    /// Sets the entire `WITH` clause at once.
    pub fn with_ctes(mut self, ctes: impl IntoIterator<Item = Cte>) -> Self {
        self.with = ctes.into_iter().collect();
        self
    }

    /// Replaces the projection list.
    pub fn projections(mut self, projections: impl IntoIterator<Item = Projection>) -> Self {
        self.projections = projections.into_iter().collect();
        self
    }

    /// Appends a single projection.
    pub fn projection(mut self, projection: Projection) -> Self {
        self.projections.push(projection);
        self
    }

    /// Sets the WHERE clause.
    pub fn where_clause(mut self, condition: ConditionExpression) -> Self {
        self.where_clause = Some(condition);
        self
    }

    /// Appends a join.
    pub fn join(mut self, join: Join) -> Self {
        self.joins.push(join);
        self
    }

    /// Sets the GROUP BY column list.
    pub fn group_by(mut self, columns: impl IntoIterator<Item = impl Into<ColumnRef>>) -> Self {
        self.group_by = columns.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the HAVING clause.
    pub fn having(mut self, condition: ConditionExpression) -> Self {
        self.having = Some(condition);
        self
    }

    /// Replaces the ORDER BY list.
    pub fn set_order_by(mut self, order_by: impl IntoIterator<Item = OrderBy>) -> Self {
        self.order_by = order_by.into_iter().collect();
        self
    }

    /// Appends a single [`OrderBy`](crate::ast::OrderBy) directive.
    pub fn order_by(mut self, order_by: OrderBy) -> Self {
        self.order_by.push(order_by);
        self
    }

    /// Sets the LIMIT.
    pub fn limit(mut self, limit: impl Into<u64>) -> Self {
        self.limit = Some(limit.into());
        self
    }

    /// Sets the OFFSET.
    pub fn offset(mut self, offset: impl Into<u64>) -> Self {
        self.offset = Some(offset.into());
        self
    }

    /// Adds `DISTINCT` to the query.
    pub fn distinct(mut self) -> Self {
        self.distinct = true;
        self
    }

    /// Removes `DISTINCT` from the query.
    pub fn clear_distinct(mut self) -> Self {
        self.distinct = false;
        self
    }

    /// Clears the projection list.
    pub fn clear_projections(mut self) -> Self {
        self.projections.clear();
        self
    }

    /// Clears the WHERE clause.
    pub fn clear_where(mut self) -> Self {
        self.where_clause = None;
        self
    }

    /// Clears all joins.
    pub fn clear_joins(mut self) -> Self {
        self.joins.clear();
        self
    }

    /// Clears the GROUP BY list.
    pub fn clear_group_by(mut self) -> Self {
        self.group_by.clear();
        self
    }

    /// Clears the HAVING clause.
    pub fn clear_having(mut self) -> Self {
        self.having = None;
        self
    }

    /// Clears the ORDER BY list.
    pub fn clear_order_by(mut self) -> Self {
        self.order_by.clear();
        self
    }

    /// Clears the LIMIT.
    pub fn clear_limit(mut self) -> Self {
        self.limit = None;
        self
    }

    /// Clears the OFFSET.
    pub fn clear_offset(mut self) -> Self {
        self.offset = None;
        self
    }

    /// Clears the WITH clause.
    pub fn clear_with(mut self) -> Self {
        self.with.clear();
        self
    }
}
