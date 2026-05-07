use crate::column::IntoColumnRef;

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
    pub fn delete(column: impl IntoColumnRef, interval: Interval) -> Self {
        Self { column: column.into_column_ref(), interval, action: TtlAction::Delete }
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

    /// Creates an [`Interval`](crate::ast::Interval) of the given number of seconds.
    pub fn seconds(value: u32) -> Self {
        Self::new(value, IntervalUnit::Second)
    }

    /// Creates an [`Interval`](crate::ast::Interval) of the given number of minutes.
    pub fn minutes(value: u32) -> Self {
        Self::new(value, IntervalUnit::Minute)
    }

    /// Creates an [`Interval`](crate::ast::Interval) of the given number of hours.
    pub fn hours(value: u32) -> Self {
        Self::new(value, IntervalUnit::Hour)
    }

    /// Creates an [`Interval`](crate::ast::Interval) of the given number of days.
    pub fn days(value: u32) -> Self {
        Self::new(value, IntervalUnit::Day)
    }

    /// Creates an [`Interval`](crate::ast::Interval) of the given number of weeks.
    pub fn weeks(value: u32) -> Self {
        Self::new(value, IntervalUnit::Week)
    }

    /// Creates an [`Interval`](crate::ast::Interval) of the given number of months.
    pub fn months(value: u32) -> Self {
        Self::new(value, IntervalUnit::Month)
    }

    /// Creates an [`Interval`](crate::ast::Interval) of the given number of years.
    pub fn years(value: u32) -> Self {
        Self::new(value, IntervalUnit::Year)
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
