use crate::ast::refs::ColumnRef;


/// A window function expression applied with an OVER clause.
#[derive(Debug, Clone, PartialEq)]
pub struct WindowFunction {
    pub func: WindowFunc,
    pub over: WindowSpec,
}

impl WindowFunction {
    /// Creates a new window function with the given function and window specification.
    pub fn new(func: WindowFunc, over: WindowSpec) -> Self {
        Self { func, over }
    }

    /// Creates a `ROW_NUMBER()` window function.
    pub fn row_number(over: WindowSpec) -> Self {
        Self::new(WindowFunc::RowNumber, over)
    }

    /// Creates a `RANK()` window function.
    pub fn rank(over: WindowSpec) -> Self {
        Self::new(WindowFunc::Rank, over)
    }

    /// Creates a `DENSE_RANK()` window function.
    pub fn dense_rank(over: WindowSpec) -> Self {
        Self::new(WindowFunc::DenseRank, over)
    }

    /// Creates a `LAG(column, offset, default)` window function.
    pub fn lag(
        col: impl Into<ColumnRef>,
        offset: Option<i64>,
        default: Option<crate::value::Value>,
        over: WindowSpec,
    ) -> Self {
        Self::new(WindowFunc::Lag { column: col.into(), offset, default }, over)
    }

    /// Creates a `LEAD(column, offset, default)` window function.
    pub fn lead(
        col: impl Into<ColumnRef>,
        offset: Option<i64>,
        default: Option<crate::value::Value>,
        over: WindowSpec,
    ) -> Self {
        Self::new(WindowFunc::Lead { column: col.into(), offset, default }, over)
    }

    /// Creates a `FIRST_VALUE(column)` window function.
    pub fn first_value(col: impl Into<ColumnRef>, over: WindowSpec) -> Self {
        Self::new(WindowFunc::FirstValue(col.into()), over)
    }

    /// Creates a `LAST_VALUE(column)` window function.
    pub fn last_value(col: impl Into<ColumnRef>, over: WindowSpec) -> Self {
        Self::new(WindowFunc::LastValue(col.into()), over)
    }

    /// Creates an `NTH_VALUE(column, n)` window function.
    pub fn nth_value(col: impl Into<ColumnRef>, n: u64, over: WindowSpec) -> Self {
        Self::new(WindowFunc::NthValue { column: col.into(), n }, over)
    }

    /// Creates an `NTILE(n)` window function.
    pub fn ntile(n: u64, over: WindowSpec) -> Self {
        Self::new(WindowFunc::Ntile(n), over)
    }

    /// Creates a `PERCENT_RANK()` window function.
    pub fn percent_rank(over: WindowSpec) -> Self {
        Self::new(WindowFunc::PercentRank, over)
    }

    /// Creates a `CUME_DIST()` window function.
    pub fn cume_dist(over: WindowSpec) -> Self {
        Self::new(WindowFunc::CumeDist, over)
    }
}

/// The function applied in a [`WindowFunction`](crate::ast::WindowFunction).
#[derive(Debug, Clone, PartialEq)]
pub enum WindowFunc {
    /// `ROW_NUMBER()`.
    RowNumber,
    /// `RANK()`.
    Rank,
    /// `DENSE_RANK()`.
    DenseRank,
    /// `LAG(column, offset, default)`.
    Lag {
        column: ColumnRef,
        offset: Option<i64>,
        default: Option<crate::value::Value>,
    },
    /// `LEAD(column, offset, default)`.
    Lead {
        column: ColumnRef,
        offset: Option<i64>,
        default: Option<crate::value::Value>,
    },
    /// `FIRST_VALUE(column)`.
    FirstValue(ColumnRef),
    /// `LAST_VALUE(column)`.
    LastValue(ColumnRef),
    /// `NTH_VALUE(column, n)`.
    NthValue {
        column: ColumnRef,
        n: u64,
    },
    /// `NTILE(n)`.
    Ntile(u64),
    /// `PERCENT_RANK()`.
    PercentRank,
    /// `CUME_DIST()`.
    CumeDist,
}

/// The OVER clause specifying the window for a [`WindowFunction`](crate::ast::WindowFunction).
#[derive(Debug, Clone, PartialEq)]
pub struct WindowSpec {
    pub partition_by: Vec<ColumnRef>,
    pub order_by: Vec<crate::ast::select::OrderBy>,
    pub frame: Option<WindowFrame>,
}

impl WindowSpec {
    /// Creates an empty window specification with no partitioning, ordering, or frame.
    pub fn new() -> Self {
        Self {
            partition_by: Vec::new(),
            order_by: Vec::new(),
            frame: None,
        }
    }

    /// Sets the PARTITION BY columns.
    pub fn partition_by(
        mut self,
        cols: impl IntoIterator<Item = impl Into<ColumnRef>>,
    ) -> Self {
        self.partition_by = cols.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the ORDER BY directives.
    pub fn order_by(
        mut self,
        order: impl IntoIterator<Item = crate::ast::select::OrderBy>,
    ) -> Self {
        self.order_by = order.into_iter().collect();
        self
    }

    /// Sets the window frame.
    pub fn frame(mut self, frame: WindowFrame) -> Self {
        self.frame = Some(frame);
        self
    }
}

impl Default for WindowSpec {
    fn default() -> Self {
        Self::new()
    }
}

/// A window frame bounding clause attached to a [`WindowSpec`](crate::ast::WindowSpec).
#[derive(Debug, Clone, PartialEq)]
pub struct WindowFrame {
    pub unit: FrameUnit,
    pub start: FrameBound,
    pub end: Option<FrameBound>,
}

impl WindowFrame {
    /// Creates a window frame with only a start bound.
    pub fn start(unit: FrameUnit, start: FrameBound) -> Self {
        Self { unit, start, end: None }
    }

    /// Creates a window frame with both start and end bounds (`BETWEEN ... AND ...`).
    pub fn between(unit: FrameUnit, start: FrameBound, end: FrameBound) -> Self {
        Self { unit, start, end: Some(end) }
    }
}

/// The unit for a [`WindowFrame`](crate::ast::WindowFrame).
#[derive(Debug, Clone, PartialEq)]
pub enum FrameUnit {
    /// `ROWS` frame unit.
    Rows,
    /// `RANGE` frame unit.
    Range,
    /// `GROUPS` frame unit.
    Groups,
}

/// A single boundary in a [`WindowFrame`](crate::ast::WindowFrame).
#[derive(Debug, Clone, PartialEq)]
pub enum FrameBound {
    /// `UNBOUNDED PRECEDING`.
    UnboundedPreceding,
    /// `<n> PRECEDING`.
    Preceding(u64),
    /// `CURRENT ROW`.
    CurrentRow,
    /// `<n> FOLLOWING`.
    Following(u64),
    /// `UNBOUNDED FOLLOWING`.
    UnboundedFollowing,
}
