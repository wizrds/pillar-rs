/// Defines a TTL (time-to-live) rule on a table, specifying when rows expire.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlClause {
    /// The column whose value is used to calculate row expiry.
    pub column: String,
    /// How long after the column value a row is retained.
    pub interval: Interval,
    /// What to do when a row expires.
    pub action: TtlAction,
}

impl TtlClause {
    /// Creates a [`TtlClause`](crate::ast::TtlClause) that deletes rows after the given interval.
    pub fn delete(column: impl Into<String>, interval: Interval) -> Self {
        Self { column: column.into(), interval, action: TtlAction::Delete }
    }
}

/// A duration used in a [`TtlClause`](crate::ast::TtlClause).
#[derive(Debug, Clone, PartialEq)]
pub struct Interval {
    pub value: u32,
    pub unit: IntervalUnit,
}

impl Interval {
    /// Creates a new [`Interval`](crate::ast::Interval) with the given value and unit.
    pub fn new(value: u32, unit: IntervalUnit) -> Self {
        Self { value, unit }
    }
}

/// The time unit for an [`Interval`](crate::ast::Interval).
#[derive(Debug, Clone, PartialEq)]
pub enum IntervalUnit {
    Second,
    Minute,
    Hour,
    Day,
    Week,
    Month,
    Year,
}

/// The action taken when a row's TTL expires.
#[derive(Debug, Clone, PartialEq)]
pub enum TtlAction {
    /// The row is deleted.
    Delete,
}
