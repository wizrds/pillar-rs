use crate::{
    value::Value,
    ast::{
        duration::Interval,
        refs::ColumnRef,
        schema::ColumnType,
        window::WindowFunction,
        select::SelectStatement,
    },
};


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

/// A scalar expression used in projections, conditions, and computed columns.
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
    /// A `CAST` expression converting a value to a specific type.
    Cast {
        expr: Box<Expression>,
        to: ColumnType,
    },
    /// A window function expression with an OVER clause.
    Window(Box<WindowFunction>),
    /// A scalar subquery returning a single value.
    Subquery(Box<SelectStatement>),
    /// An interval literal.
    Interval(Interval),
    /// A time-bucketing expression that truncates a timestamp column to a fixed interval width.
    TimeBucket {
        interval: Interval,
        column: ColumnRef,
    },
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

    /// Casts this expression to the given column type.
    pub fn cast(expr: impl Into<Expression>, to: ColumnType) -> Self {
        Self::Cast { expr: Box::new(expr.into()), to }
    }

    /// Wraps a window function as an expression.
    pub fn window(func: WindowFunction) -> Self {
        Self::Window(Box::new(func))
    }

    /// Wraps a scalar subquery as an expression.
    pub fn subquery(stmt: SelectStatement) -> Self {
        Self::Subquery(Box::new(stmt))
    }

    /// An interval literal.
    pub fn interval(interval: Interval) -> Self {
        Self::Interval(interval)
    }

    /// A time-bucketing expression that truncates a timestamp column to a fixed interval width.
    pub fn time_bucket(interval: Interval, column: impl Into<ColumnRef>) -> Self {
        Self::TimeBucket { interval, column: column.into() }
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

    /// Adds a `WHEN ... THEN ...` branch.
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
