use crate::{
    condition::ConditionExpression,
    value::Value,
    ast::refs::{ColumnRef, TableRef},
};


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

/// An aggregate function used in a [`Projection`](crate::ast::Projection) or [`Expression`](crate::ast::Expression).
#[derive(Debug, Clone, PartialEq)]
pub enum AggregateFunction {
    /// `COUNT(*)`, `COUNT(column)`, or `COUNT(DISTINCT column)`.
    Count(CountArg),
    /// `SUM(column)`.
    Sum(ColumnRef),
    /// `AVG(column)`.
    Avg(ColumnRef),
    /// `MIN(column)`.
    Min(ColumnRef),
    /// `MAX(column)`.
    Max(ColumnRef),
    /// `approxCountDistinct(column)` or equivalent.
    ApproxCountDistinct(ColumnRef),
    /// `uniq(column)` or equivalent.
    Uniq(ColumnRef),
    /// `QUANTILE(level)(column)` or equivalent.
    Quantile { level: f64, column: ColumnRef },
    /// `topK(k)(column)` or equivalent.
    TopK { k: u32, column: ColumnRef },
    /// `histogram(bins)(column)` or equivalent.
    Histogram { bins: u32, column: ColumnRef },
    /// Wraps a function to produce its intermediate state (e.g. `countState`).
    State(Box<AggregateFunction>),
    /// Merges intermediate aggregate states (e.g. `countMerge`).
    Merge(Box<AggregateFunction>),
}

impl AggregateFunction {
    /// `COUNT(*)`.
    pub fn count_all() -> Self {
        Self::Count(CountArg::All)
    }

    /// `COUNT(column)`.
    pub fn count(col: impl Into<ColumnRef>) -> Self {
        Self::Count(CountArg::Column(col.into()))
    }

    /// `COUNT(DISTINCT column)`.
    pub fn count_distinct(col: impl Into<ColumnRef>) -> Self {
        Self::Count(CountArg::Distinct(col.into()))
    }

    /// `SUM(column)`.
    pub fn sum(col: impl Into<ColumnRef>) -> Self {
        Self::Sum(col.into())
    }

    /// `AVG(column)`.
    pub fn avg(col: impl Into<ColumnRef>) -> Self {
        Self::Avg(col.into())
    }

    /// `MIN(column)`.
    pub fn min(col: impl Into<ColumnRef>) -> Self {
        Self::Min(col.into())
    }

    /// `MAX(column)`.
    pub fn max(col: impl Into<ColumnRef>) -> Self {
        Self::Max(col.into())
    }

    /// `approxCountDistinct(column)` or equivalent.
    pub fn approx_count_distinct(col: impl Into<ColumnRef>) -> Self {
        Self::ApproxCountDistinct(col.into())
    }

    /// `uniq(column)` or equivalent.
    pub fn uniq(col: impl Into<ColumnRef>) -> Self {
        Self::Uniq(col.into())
    }

    /// `QUANTILE(level)(column)` or equivalent.
    pub fn quantile(level: f64, col: impl Into<ColumnRef>) -> Self {
        Self::Quantile { level, column: col.into() }
    }

    /// `topK(k)(column)` or equivalent.
    pub fn top_k(k: u32, col: impl Into<ColumnRef>) -> Self {
        Self::TopK { k, column: col.into() }
    }

    /// `histogram(bins)(column)` or equivalent.
    pub fn histogram(bins: u32, col: impl Into<ColumnRef>) -> Self {
        Self::Histogram { bins, column: col.into() }
    }

    /// Wraps this function to produce its intermediate aggregate state.
    pub fn state(inner: impl Into<AggregateFunction>) -> Self {
        Self::State(Box::new(inner.into()))
    }

    /// Merges the intermediate aggregate state produced by [`state`](AggregateFunction::state).
    pub fn merge(inner: impl Into<AggregateFunction>) -> Self {
        Self::Merge(Box::new(inner.into()))
    }
}

/// The argument to a [`Count`](crate::ast::AggregateFunction::Count) aggregate.
#[derive(Debug, Clone, PartialEq)]
pub enum CountArg {
    /// `COUNT(*)`.
    All,
    /// `COUNT(column)`.
    Column(ColumnRef),
    /// `COUNT(DISTINCT column)`.
    Distinct(ColumnRef),
}

impl CountArg {
    /// `COUNT(*)`.
    pub fn all() -> Self {
        Self::All
    }

    /// `COUNT(column)`.
    pub fn column(col: impl Into<ColumnRef>) -> Self {
        Self::Column(col.into())
    }

    /// `COUNT(DISTINCT column)`.
    pub fn distinct(col: impl Into<ColumnRef>) -> Self {
        Self::Distinct(col.into())
    }
}

/// A scalar expression used in projections and computed columns.
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    /// A literal value.
    Value(Value),
    /// A column reference.
    Column(ColumnRef),
    /// A binary arithmetic or string operation.
    BinaryOp {
        left: Box<Expression>,
        op: BinaryOperator,
        right: Box<Expression>,
    },
    /// A scalar function call.
    Function {
        name: String,
        args: Vec<Expression>,
    },
    /// A `CASE` expression with optional operand and `ELSE` branch.
    Case {
        operand: Option<Box<Expression>>,
        when_then: Vec<(Expression, Expression)>,
        else_result: Option<Box<Expression>>,
    },
    /// An aggregate function used as a scalar expression.
    Aggregate(AggregateFunction),
}

impl Expression {
    /// A literal value.
    pub fn value(v: impl Into<Value>) -> Self {
        Self::Value(v.into())
    }

    /// A column reference.
    pub fn column(col: impl Into<ColumnRef>) -> Self {
        Self::Column(col.into())
    }

    /// A scalar function call.
    pub fn function(
        name: impl Into<String>,
        args: impl IntoIterator<Item = impl Into<Expression>>,
    ) -> Self {
        Self::Function {
            name: name.into(),
            args: args.into_iter().map(Into::into).collect(),
        }
    }

    /// An aggregate function expression.
    pub fn aggregate(f: AggregateFunction) -> Self {
        Self::Aggregate(f)
    }

    /// Starts building a `CASE` expression.
    pub fn case() -> CaseBuilder {
        CaseBuilder::new()
    }

    /// Adds this expression to `rhs` (`+`).
    pub fn add(self, rhs: impl Into<Expression>) -> Self {
        Self::BinaryOp {
            left: Box::new(self),
            op: BinaryOperator::Add,
            right: Box::new(rhs.into()),
        }
    }

    /// Subtracts `rhs` from this expression (`-`).
    pub fn subtract(self, rhs: impl Into<Expression>) -> Self {
        Self::BinaryOp {
            left: Box::new(self),
            op: BinaryOperator::Subtract,
            right: Box::new(rhs.into()),
        }
    }

    /// Multiplies this expression by `rhs` (`*`).
    pub fn multiply(self, rhs: impl Into<Expression>) -> Self {
        Self::BinaryOp {
            left: Box::new(self),
            op: BinaryOperator::Multiply,
            right: Box::new(rhs.into()),
        }
    }

    /// Divides this expression by `rhs` (`/`).
    pub fn divide(self, rhs: impl Into<Expression>) -> Self {
        Self::BinaryOp {
            left: Box::new(self),
            op: BinaryOperator::Divide,
            right: Box::new(rhs.into()),
        }
    }

    /// Computes this expression modulo `rhs` (`%`).
    pub fn modulo(self, rhs: impl Into<Expression>) -> Self {
        Self::BinaryOp {
            left: Box::new(self),
            op: BinaryOperator::Modulo,
            right: Box::new(rhs.into()),
        }
    }

    /// Concatenates this expression with `rhs` (`||`).
    pub fn concat(self, rhs: impl Into<Expression>) -> Self {
        Self::BinaryOp {
            left: Box::new(self),
            op: BinaryOperator::Concat,
            right: Box::new(rhs.into()),
        }
    }
}

impl From<Value> for Expression {
    fn from(v: Value) -> Self {
        Self::Value(v)
    }
}

/// Builder for a `CASE` expression.
#[derive(Debug, Clone, PartialEq)]
pub struct CaseBuilder {
    operand: Option<Box<Expression>>,
    when_then: Vec<(Expression, Expression)>,
}

impl CaseBuilder {
    fn new() -> Self {
        Self { operand: None, when_then: Vec::new() }
    }

    /// Sets the `CASE` operand for a simple (non-searched) `CASE`.
    pub fn operand(mut self, expr: impl Into<Expression>) -> Self {
        self.operand = Some(Box::new(expr.into()));
        self
    }

    /// Adds a `WHEN … THEN …` branch.
    pub fn when(mut self, when: impl Into<Expression>, then: impl Into<Expression>) -> Self {
        self.when_then.push((when.into(), then.into()));
        self
    }

    /// Finalizes the `CASE` with an `ELSE` branch.
    pub fn otherwise(self, else_result: impl Into<Expression>) -> Expression {
        Expression::Case {
            operand: self.operand,
            when_then: self.when_then,
            else_result: Some(Box::new(else_result.into())),
        }
    }

    /// Finalizes the `CASE` without an `ELSE` branch.
    pub fn build(self) -> Expression {
        Expression::Case {
            operand: self.operand,
            when_then: self.when_then,
            else_result: None,
        }
    }
}

impl From<CaseBuilder> for Expression {
    fn from(b: CaseBuilder) -> Self {
        b.build()
    }
}

/// A binary arithmetic or string operator in an [`Expression`](crate::ast::Expression).
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    /// Addition (`+`).
    Add,
    /// Subtraction (`-`).
    Subtract,
    /// Multiplication (`*`).
    Multiply,
    /// Division (`/`).
    Divide,
    /// Modulo (`%`).
    Modulo,
    /// String concatenation (`||`).
    Concat,
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

/// AST node for a `SELECT` statement.
#[derive(Debug, Clone, PartialEq)]
pub struct SelectStatement {
    pub distinct: bool,
    pub projections: Vec<Projection>,
    pub from: TableRef,
    pub joins: Vec<Join>,
    pub where_clause: Option<ConditionExpression>,
    pub group_by: Vec<ColumnRef>,
    pub having: Option<ConditionExpression>,
    pub order_by: Vec<OrderBy>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

impl SelectStatement {
    /// Creates a new [`SelectStatement`](crate::ast::SelectStatement) selecting all columns from the given table.
    pub fn new(from: impl Into<TableRef>) -> Self {
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
        }
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
}
