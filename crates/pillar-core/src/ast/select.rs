use crate::{
    condition::ConditionExpression,
    value::Value,
    ast::table::TableRef,
};


/// A column projection in a [`SelectStatement`](crate::ast::SelectStatement).
#[derive(Debug, Clone, PartialEq)]
pub enum Projection {
    /// Selects all columns (`*`).
    All,
    /// Selects a single named column.
    Column(String),
    /// Selects a column with an alias.
    ColumnAlias(String, String),
    /// Selects an aggregate expression.
    Aggregate(AggregateFunction),
    /// Selects an arbitrary expression.
    Expression(Expression),
}

/// An aggregate function used in a [`Projection`](crate::ast::Projection) or [`Expression`](crate::ast::Expression).
#[derive(Debug, Clone, PartialEq)]
pub enum AggregateFunction {
    Count(CountArg),
    Sum(String),
    Avg(String),
    Min(String),
    Max(String),
    ApproxCountDistinct(String),
    Uniq(String),
    Quantile { level: f64, column: String },
    TopK { k: u32, column: String },
    Histogram { bins: u32, column: String },
    /// Wraps a function to produce its intermediate state (e.g. `countState`).
    State(Box<AggregateFunction>),
    /// Merges intermediate aggregate states (e.g. `countMerge`).
    Merge(Box<AggregateFunction>),
}

/// The argument to a [`Count`](crate::ast::AggregateFunction::Count) aggregate.
#[derive(Debug, Clone, PartialEq)]
pub enum CountArg {
    /// `COUNT(*)`.
    All,
    /// `COUNT(column)`.
    Column(String),
    /// `COUNT(DISTINCT column)`.
    Distinct(String),
}

/// A scalar expression used in projections and computed columns.
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Value(Value),
    Column(String),
    BinaryOp {
        left: Box<Expression>,
        op: BinaryOperator,
        right: Box<Expression>,
    },
    Function {
        name: String,
        args: Vec<Expression>,
    },
    Case {
        operand: Option<Box<Expression>>,
        when_then: Vec<(Expression, Expression)>,
        else_result: Option<Box<Expression>>,
    },
    Aggregate(AggregateFunction),
}

/// A binary arithmetic or string operator in an [`Expression`](crate::ast::Expression).
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Concat,
}

/// A join clause in a [`SelectStatement`](crate::ast::SelectStatement).
#[derive(Debug, Clone, PartialEq)]
pub struct Join {
    pub table: TableRef,
    pub on: ConditionExpression,
    pub join_type: JoinType,
}

/// The type of join to perform.
#[derive(Debug, Clone, PartialEq)]
pub enum JoinType {
    Inner,
    Left,
    Right,
    Full,
    Cross,
}

/// A single column ordering directive in a [`SelectStatement`](crate::ast::SelectStatement).
#[derive(Debug, Clone, PartialEq)]
pub struct OrderBy {
    pub column: String,
    pub direction: OrderDirection,
    pub nulls: Option<NullsOrder>,
}

impl OrderBy {
    /// Orders by the given column ascending.
    pub fn asc(column: impl Into<String>) -> Self {
        Self { column: column.into(), direction: OrderDirection::Asc, nulls: None }
    }

    /// Orders by the given column descending.
    pub fn desc(column: impl Into<String>) -> Self {
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
    Asc,
    Desc,
}

/// Whether nulls appear first or last in an [`OrderBy`](crate::ast::OrderBy).
#[derive(Debug, Clone, PartialEq)]
pub enum NullsOrder {
    First,
    Last,
}

/// AST node for a `SELECT` statement.
#[derive(Debug, Clone, PartialEq)]
pub struct SelectStatement {
    pub distinct: bool,
    pub projections: Vec<Projection>,
    pub from: TableRef,
    pub joins: Vec<Join>,
    pub where_clause: Option<ConditionExpression>,
    pub group_by: Vec<String>,
    pub having: Option<ConditionExpression>,
    pub order_by: Vec<OrderBy>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

impl SelectStatement {
    /// Creates a new [`SelectStatement`](crate::ast::SelectStatement) selecting all columns from the given table.
    pub fn new(from: TableRef) -> Self {
        Self {
            distinct: false,
            projections: vec![Projection::All],
            from,
            joins: Vec::new(),
            where_clause: None,
            group_by: Vec::new(),
            having: None,
            order_by: Vec::new(),
            limit: None,
            offset: None,
        }
    }

    /// Replaces the projection list.
    pub fn projections(mut self, projections: Vec<Projection>) -> Self {
        self.projections = projections;
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
    pub fn group_by(mut self, columns: Vec<String>) -> Self {
        self.group_by = columns;
        self
    }

    /// Sets the HAVING clause.
    pub fn having(mut self, condition: ConditionExpression) -> Self {
        self.having = Some(condition);
        self
    }

    /// Replaces the ORDER BY list.
    pub fn set_order_by(mut self, order_by: Vec<OrderBy>) -> Self {
        self.order_by = order_by;
        self
    }

    /// Appends a single [`OrderBy`](crate::ast::OrderBy) directive.
    pub fn order_by(mut self, order_by: OrderBy) -> Self {
        self.order_by.push(order_by);
        self
    }

    /// Sets the LIMIT.
    pub fn limit(mut self, limit: u64) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Sets the OFFSET.
    pub fn offset(mut self, offset: u64) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Adds `DISTINCT` to the query.
    pub fn distinct(mut self) -> Self {
        self.distinct = true;
        self
    }
}
