#[derive(Debug, Clone, PartialEq)]
pub struct TtlClause {
    pub column: String,
    pub interval: Interval,
    pub action: TtlAction,
}

impl TtlClause {
    pub fn delete(column: impl Into<String>, interval: Interval) -> Self {
        Self { column: column.into(), interval, action: TtlAction::Delete }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Interval {
    pub value: u32,
    pub unit: IntervalUnit,
}

impl Interval {
    pub fn new(value: u32, unit: IntervalUnit) -> Self {
        Self { value, unit }
    }
}

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

#[derive(Debug, Clone, PartialEq)]
pub enum TtlAction {
    Delete,
}
