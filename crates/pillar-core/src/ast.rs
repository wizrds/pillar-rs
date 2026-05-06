use crate::{value::Value, condition::ConditionExpression};


#[derive(Debug, Clone, PartialEq)]
pub struct ColumnDefinition {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub primary_key: bool,
    pub default: Option<Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TableRef {
    pub name: String,
    pub alias: Option<String>,
}

impl TableRef {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            alias: None,
        }
    }

    pub fn alias(mut self, alias: impl Into<String>) -> Self {
        self.alias = Some(alias.into());
        self
    }
}

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
        Self {
            column: column.into(),
            direction: OrderDirection::Asc,
            nulls: None,
        }
    }

    pub fn desc(column: impl Into<String>) -> Self {
        Self {
            column: column.into(),
            direction: OrderDirection::Desc,
            nulls: None,
        }
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
pub struct OnConflict {
    pub target: Vec<String>,
    pub action: OnConflictAction,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OnConflictAction {
    DoNothing,
    DoUpdate {
        set: Vec<(String, Value)>,
        where_clause: Option<ConditionExpression>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Select(SelectStatement),
    Insert(InsertStatement),
    Update(UpdateStatement),
    Delete(DeleteStatement),
    CreateTable(CreateTableStatement),
    AlterTable(AlterTableStatement),
    DropTable(DropTableStatement),
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

    pub fn order_by(mut self, order_by: Vec<OrderBy>) -> Self {
        self.order_by = order_by;
        self
    }

    pub fn order_by_column(mut self, order_by: OrderBy) -> Self {
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

#[derive(Debug, Clone, PartialEq)]
pub struct InsertStatement {
    pub table: TableRef,
    pub columns: Vec<String>,
    pub values: Vec<Value>,
    pub on_conflict: Option<OnConflict>,
}

impl InsertStatement {
    pub fn new(table: TableRef) -> Self {
        Self {
            table,
            columns: Vec::new(),
            values: Vec::new(),
            on_conflict: None,
        }
    }

    pub fn columns(mut self, columns: Vec<String>) -> Self {
        self.columns = columns;
        self
    }

    pub fn values(mut self, values: Vec<Value>) -> Self {
        self.values = values;
        self
    }

    pub fn on_conflict(mut self, on_conflict: OnConflict) -> Self {
        self.on_conflict = Some(on_conflict);
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UpdateStatement {
    pub table: TableRef,
    pub set: Vec<(String, Value)>,
    pub where_clause: Option<ConditionExpression>,
}

impl UpdateStatement {
    pub fn new(table: TableRef) -> Self {
        Self {
            table,
            set: Vec::new(),
            where_clause: None,
        }
    }

    pub fn set(mut self, set: Vec<(String, Value)>) -> Self {
        self.set = set;
        self
    }

    pub fn where_clause(mut self, condition: ConditionExpression) -> Self {
        self.where_clause = Some(condition);
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeleteStatement {
    pub table: TableRef,
    pub where_clause: Option<ConditionExpression>,
}

impl DeleteStatement {
    pub fn new(table: TableRef) -> Self {
        Self {
            table,
            where_clause: None,
        }
    }

    pub fn where_clause(mut self, condition: ConditionExpression) -> Self {
        self.where_clause = Some(condition);
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateTableStatement {
    pub name: String,
    pub columns: Vec<ColumnDefinition>,
    pub if_not_exists: bool,
    pub engine: Option<String>,
    pub partition_by: Option<String>,
}

impl CreateTableStatement {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            columns: Vec::new(),
            if_not_exists: false,
            engine: None,
            partition_by: None,
        }
    }

    pub fn columns(mut self, columns: Vec<ColumnDefinition>) -> Self {
        self.columns = columns;
        self
    }

    pub fn if_not_exists(mut self) -> Self {
        self.if_not_exists = true;
        self
    }

    pub fn engine(mut self, engine: impl Into<String>) -> Self {
        self.engine = Some(engine.into());
        self
    }

    pub fn partition_by(mut self, partition_by: impl Into<String>) -> Self {
        self.partition_by = Some(partition_by.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlterTableStatement {
    pub name: String,
    pub add_columns: Vec<ColumnDefinition>,
    pub drop_columns: Vec<String>,
}

impl AlterTableStatement {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            add_columns: Vec::new(),
            drop_columns: Vec::new(),
        }
    }

    pub fn add_columns(mut self, columns: Vec<ColumnDefinition>) -> Self {
        self.add_columns = columns;
        self
    }

    pub fn drop_columns(mut self, columns: Vec<String>) -> Self {
        self.drop_columns = columns;
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DropTableStatement {
    pub name: String,
    pub if_exists: bool,
}

impl DropTableStatement {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            if_exists: false,
        }
    }

    pub fn if_exists(mut self) -> Self {
        self.if_exists = true;
        self
    }
}
