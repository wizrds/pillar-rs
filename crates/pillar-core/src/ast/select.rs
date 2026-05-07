use crate::{
    condition::ConditionExpression,
    value::Value,
    ast::table::TableRef,
};


#[derive(Debug, Clone, PartialEq)]
pub enum Projection {
    All,
    Column(String),
    ColumnAlias(String, String),
    Aggregate(AggregateFunction),
    Expression(Expression),
}

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
    State(Box<AggregateFunction>),
    Merge(Box<AggregateFunction>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CountArg {
    All,
    Column(String),
    Distinct(String),
}

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

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Concat,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Join {
    pub table: TableRef,
    pub on: ConditionExpression,
    pub join_type: JoinType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum JoinType {
    Inner,
    Left,
    Right,
    Full,
    Cross,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OrderBy {
    pub column: String,
    pub direction: OrderDirection,
    pub nulls: Option<NullsOrder>,
}

impl OrderBy {
    pub fn asc(column: impl Into<String>) -> Self {
        Self { column: column.into(), direction: OrderDirection::Asc, nulls: None }
    }

    pub fn desc(column: impl Into<String>) -> Self {
        Self { column: column.into(), direction: OrderDirection::Desc, nulls: None }
    }

    pub fn nulls(mut self, nulls: NullsOrder) -> Self {
        self.nulls = Some(nulls);
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum OrderDirection {
    Asc,
    Desc,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NullsOrder {
    First,
    Last,
}

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

    pub fn projections(mut self, projections: Vec<Projection>) -> Self {
        self.projections = projections;
        self
    }

    pub fn projection(mut self, projection: Projection) -> Self {
        self.projections.push(projection);
        self
    }

    pub fn where_clause(mut self, condition: ConditionExpression) -> Self {
        self.where_clause = Some(condition);
        self
    }

    pub fn join(mut self, join: Join) -> Self {
        self.joins.push(join);
        self
    }

    pub fn group_by(mut self, columns: Vec<String>) -> Self {
        self.group_by = columns;
        self
    }

    pub fn having(mut self, condition: ConditionExpression) -> Self {
        self.having = Some(condition);
        self
    }

    pub fn set_order_by(mut self, order_by: Vec<OrderBy>) -> Self {
        self.order_by = order_by;
        self
    }

    pub fn order_by(mut self, order_by: OrderBy) -> Self {
        self.order_by.push(order_by);
        self
    }

    pub fn limit(mut self, limit: u64) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn offset(mut self, offset: u64) -> Self {
        self.offset = Some(offset);
        self
    }

    pub fn distinct(mut self) -> Self {
        self.distinct = true;
        self
    }
}
