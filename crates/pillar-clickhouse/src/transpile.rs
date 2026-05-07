use pillar_core::{
    ast::{
        AggregateFunction, AlterTableStatement, BinaryOperator, ColumnDefinition, CountArg,
        CreateMaterializedViewStatement, CreateTableStatement, CreateViewStatement,
        DeleteStatement, DropTableStatement, DropViewStatement, Expression, InsertStatement,
        IntervalUnit, JoinType, NullsOrder, OrderDirection, Projection, SelectStatement,
        Statement, UpdateStatement, AggregateFn, ColumnType
    },
    condition::ConditionExpression,
    dialect::PreparedStatement,
    errors::Error,
    value::{ToSql, Value},
};


pub struct Transpiler {
    params: Vec<Value>,
    count: usize,
}

impl Transpiler {
    pub fn new() -> Self {
        Self { params: Vec::new(), count: 0 }
    }

    fn placeholder(&mut self, value: Value, inline: bool) -> String {
        if inline {
            return value.to_sql();
        }
        self.params.push(value);
        self.count += 1;
        "?".to_string()
    }

    fn select(&mut self, stmt: &SelectStatement, inline: bool) -> Result<String, Error> {
        let mut sql = format!(
            "{} {} FROM {}",
            if stmt.distinct { "SELECT DISTINCT" } else { "SELECT" },
            stmt.projections
                .iter()
                .map(|p| self.projection(p, inline))
                .collect::<Vec<_>>()
                .join(", "),
            match &stmt.from.alias {
                Some(alias) => format!("{} AS {}", stmt.from.name, alias),
                None => stmt.from.name.clone(),
            },
        );

        for join in &stmt.joins {
            sql.push_str(&format!(
                " {} {} ON {}",
                match join.join_type {
                    JoinType::Inner => "INNER JOIN",
                    JoinType::Left => "LEFT JOIN",
                    JoinType::Right => "RIGHT JOIN",
                    JoinType::Full => "FULL JOIN",
                    JoinType::Cross => "CROSS JOIN",
                },
                match &join.table.alias {
                    Some(alias) => format!("{} AS {}", join.table.name, alias),
                    None => join.table.name.clone(),
                },
                self.condition(&join.on, inline),
            ));
        }

        if let Some(where_clause) = &stmt.where_clause {
            sql.push_str(&format!(" WHERE {}", self.condition(where_clause, inline)));
        }

        if !stmt.group_by.is_empty() {
            sql.push_str(&format!(" GROUP BY {}", stmt.group_by.join(", ")));
        }

        if let Some(having) = &stmt.having {
            sql.push_str(&format!(" HAVING {}", self.condition(having, inline)));
        }

        if !stmt.order_by.is_empty() {
            sql.push_str(&format!(
                " ORDER BY {}",
                stmt.order_by
                    .iter()
                    .map(|o| format!(
                        "{} {}{}",
                        o.column,
                        match o.direction {
                            OrderDirection::Asc => "ASC",
                            OrderDirection::Desc => "DESC",
                        },
                        match &o.nulls {
                            Some(NullsOrder::First) => " NULLS FIRST",
                            Some(NullsOrder::Last) => " NULLS LAST",
                            None => "",
                        },
                    ))
                    .collect::<Vec<_>>()
                    .join(", "),
            ));
        }

        if let Some(limit) = stmt.limit {
            sql.push_str(&format!(" LIMIT {limit}"));
        }

        if let Some(offset) = stmt.offset {
            sql.push_str(&format!(" OFFSET {offset}"));
        }

        Ok(sql)
    }

    fn insert(&mut self, stmt: &InsertStatement) -> Result<String, Error> {
        if stmt.values.is_empty() {
            return Err(Error::invalid_query("INSERT statement has no rows"));
        }

        let col_count = stmt.columns.len();

        for (i, row) in stmt.values.iter().enumerate() {
            if row.len() != col_count {
                return Err(Error::invalid_query(format!(
                    "row {i} has {} values but {col_count} columns were specified",
                    row.len(),
                )));
            }
        }

        Ok(format!(
            "INSERT INTO {} ({}) VALUES {}",
            stmt.table.name,
            stmt.columns.join(", "),
            stmt.values
                .iter()
                .map(|row| format!(
                    "({})",
                    row.iter()
                        .map(|v| self.placeholder(v.clone(), false))
                        .collect::<Vec<_>>()
                        .join(", "),
                ))
                .collect::<Vec<_>>()
                .join(", "),
        ))
    }

    fn update(&mut self, stmt: &UpdateStatement) -> Result<String, Error> {
        if stmt.set.is_empty() {
            return Err(Error::invalid_query("UPDATE statement has no SET clauses"));
        }

        let mut sql = format!(
            "ALTER TABLE {} UPDATE {}",
            stmt.table.name,
            stmt.set
                .iter()
                .map(|(col, val)| format!("{col} = {}", self.placeholder(val.clone(), false)))
                .collect::<Vec<_>>()
                .join(", "),
        );

        if let Some(where_clause) = &stmt.where_clause {
            sql.push_str(&format!(" WHERE {}", self.condition(where_clause, false)));
        }

        Ok(sql)
    }

    fn delete(&mut self, stmt: &DeleteStatement) -> Result<String, Error> {
        let mut sql = format!("ALTER TABLE {} DELETE", stmt.table.name);

        if let Some(where_clause) = &stmt.where_clause {
            sql.push_str(&format!(" WHERE {}", self.condition(where_clause, false)));
        }

        Ok(sql)
    }

    fn create_table(&mut self, stmt: &CreateTableStatement) -> Result<String, Error> {
        let mut sql = format!(
            "CREATE TABLE{} {} ({})",
            if stmt.if_not_exists { " IF NOT EXISTS" } else { "" },
            stmt.name,
            stmt.columns
                .iter()
                .map(|col| self.column_definition(col))
                .collect::<Vec<_>>()
                .join(", "),
        );

        if let Some(engine) = stmt.options.get("engine") {
            sql.push_str(&format!(" ENGINE = {engine}"));
        }

        if let Some(order_by) = stmt.options.get("order_by") {
            sql.push_str(&format!(" ORDER BY {order_by}"));
        }

        if let Some(partition_by) = stmt.options.get("partition_by") {
            sql.push_str(&format!(" PARTITION BY {partition_by}"));
        }

        if let Some(primary_key) = stmt.options.get("primary_key") {
            sql.push_str(&format!(" PRIMARY KEY {primary_key}"));
        }

        if let Some(settings) = stmt.options.get("settings") {
            sql.push_str(&format!(" SETTINGS {settings}"));
        }

        if let Some(ttl) = &stmt.ttl {
            sql.push_str(&format!(" TTL {} + INTERVAL {} {}", ttl.column, ttl.interval.value, self.interval_unit(&ttl.interval.unit)));
        }

        Ok(sql)
    }

    fn interval_unit(&self, unit: &IntervalUnit) -> &'static str {
        match unit {
            IntervalUnit::Second => "SECOND",
            IntervalUnit::Minute => "MINUTE",
            IntervalUnit::Hour => "HOUR",
            IntervalUnit::Day => "DAY",
            IntervalUnit::Week => "WEEK",
            IntervalUnit::Month => "MONTH",
            IntervalUnit::Year => "YEAR",
        }
    }

    fn alter_table(&mut self, stmt: &AlterTableStatement) -> Result<String, Error> {
        let parts = stmt
            .add_columns
            .iter()
            .map(|col| format!("ADD COLUMN {}", self.column_definition(col)))
            .chain(
                stmt.drop_columns
                    .iter()
                    .map(|col| format!("DROP COLUMN {col}")),
            )
            .collect::<Vec<_>>();

        if parts.is_empty() && stmt.ttl.is_none() {
            return Err(Error::invalid_query("ALTER TABLE statement has no operations"));
        }

        let mut sql = format!("ALTER TABLE {} {}", stmt.name, parts.join(", "));

        if let Some(ttl) = &stmt.ttl {
            if !parts.is_empty() {
                sql.push(',');
            }
            sql.push_str(&format!(" MODIFY TTL {} + INTERVAL {} {}", ttl.column, ttl.interval.value, self.interval_unit(&ttl.interval.unit)));
        }

        Ok(sql)
    }

    fn drop_table(&mut self, stmt: &DropTableStatement) -> Result<String, Error> {
        Ok(format!(
            "DROP TABLE{} {}",
            if stmt.if_exists { " IF EXISTS" } else { "" },
            stmt.name,
        ))
    }

    fn create_view(&mut self, stmt: &CreateViewStatement) -> Result<String, Error> {
        Ok(format!(
            "CREATE{} VIEW{} {} AS {}",
            if stmt.or_replace { " OR REPLACE" } else { "" },
            if stmt.if_not_exists { " IF NOT EXISTS" } else { "" },
            stmt.name,
            self.select(&stmt.query, true)?,
        ))
    }

    fn create_materialized_view(&mut self, stmt: &CreateMaterializedViewStatement) -> Result<String, Error> {
        let mut sql = format!(
            "CREATE MATERIALIZED VIEW{}{} {}",
            if stmt.or_replace { " OR REPLACE" } else { "" },
            if stmt.if_not_exists { " IF NOT EXISTS" } else { "" },
            stmt.name,
        );

        if let Some(to_table) = &stmt.to_table {
            sql.push_str(&format!(" TO {to_table}"));
        }

        if let Some(engine) = stmt.options.get("engine") {
            sql.push_str(&format!(" ENGINE = {engine}"));
        }

        if stmt.options.get("populate").map(String::as_str) == Some("true") {
            sql.push_str(" POPULATE");
        }

        sql.push_str(&format!(" AS {}", self.select(&stmt.query, true)?));

        Ok(sql)
    }

    fn drop_view(&mut self, stmt: &DropViewStatement) -> Result<String, Error> {
        Ok(format!(
            "DROP {}{} {}",
            if stmt.materialized { "TABLE" } else { "VIEW" },
            if stmt.if_exists { " IF EXISTS" } else { "" },
            stmt.name,
        ))
    }

    fn column_type(&self, col_type: &ColumnType) -> String {
        match col_type {
            ColumnType::Boolean => "Bool".to_string(),
            ColumnType::Int8 => "Int8".to_string(),
            ColumnType::Int16 => "Int16".to_string(),
            ColumnType::Int32 => "Int32".to_string(),
            ColumnType::Int64 => "Int64".to_string(),
            ColumnType::UInt8 => "UInt8".to_string(),
            ColumnType::UInt16 => "UInt16".to_string(),
            ColumnType::UInt32 => "UInt32".to_string(),
            ColumnType::UInt64 => "UInt64".to_string(),
            ColumnType::Float32 => "Float32".to_string(),
            ColumnType::Float64 => "Float64".to_string(),
            ColumnType::String => "String".to_string(),
            ColumnType::Binary => "String".to_string(),
            ColumnType::List(inner) => format!("Array({})", self.column_type(inner)),
            ColumnType::Map(k, v) => format!("Map({}, {})", self.column_type(k), self.column_type(v)),
            #[cfg(feature = "chrono")]
            ColumnType::Date => "Date".to_string(),
            #[cfg(feature = "chrono")]
            ColumnType::Time => "String".to_string(),
            #[cfg(feature = "chrono")]
            ColumnType::DateTime => "DateTime".to_string(),
            #[cfg(feature = "uuid")]
            ColumnType::Uuid => "UUID".to_string(),
            ColumnType::DateTime64 { precision } => format!("DateTime64({precision})"),
            ColumnType::LowCardinalityString => "LowCardinality(String)".to_string(),
            ColumnType::FixedString(n) => format!("FixedString({n})"),
            ColumnType::AggregateState(state) => {
                let fn_name = match &state.function {
                    AggregateFn::Count => "count".to_string(),
                    AggregateFn::Sum => "sum".to_string(),
                    AggregateFn::Avg => "avg".to_string(),
                    AggregateFn::Min => "min".to_string(),
                    AggregateFn::Max => "max".to_string(),
                    AggregateFn::Uniq => "uniq".to_string(),
                    AggregateFn::Quantile(level) => format!("quantile({level})"),
                    AggregateFn::TopK(k) => format!("topK({k})"),
                    AggregateFn::Histogram(bins) => format!("histogram({bins})"),
                    AggregateFn::Custom(name) => name.clone(),
                };
                let arg_types = state.arg_types.iter()
                    .map(|t| self.column_type(t))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("AggregateFunction({fn_name}, {arg_types})")
            }
            ColumnType::Nullable(inner) => format!("Nullable({})", self.column_type(inner)),
            ColumnType::Custom(s) => s.clone(),
        }
    }

    fn column_definition(&self, col: &ColumnDefinition) -> String {
        let type_str = if col.nullable {
            format!("Nullable({})", self.column_type(&col.data_type))
        } else {
            self.column_type(&col.data_type)
        };

        format!(
            "{} {}{}{}",
            col.name,
            type_str,
            if col.primary_key { " PRIMARY KEY" } else { "" },
            match &col.default {
                Some(val) => format!(" DEFAULT {val}"),
                None => String::new(),
            },
        )
    }

    fn projection(&mut self, proj: &Projection, inline: bool) -> String {
        match proj {
            Projection::All => "*".to_string(),
            Projection::Column(col) => col.clone(),
            Projection::ColumnAlias(col, alias) => format!("{col} AS {alias}"),
            Projection::Aggregate(agg) => self.aggregate(agg),
            Projection::Expression(expr) => self.expression(expr, inline),
            Projection::Aliased(inner, alias) => {
                format!("{} AS {alias}", self.projection(inner, inline))
            }
        }
    }

    fn expression(&mut self, expr: &Expression, inline: bool) -> String {
        match expr {
            Expression::Value(val) => self.placeholder(val.clone(), inline),
            Expression::Column(col) => col.clone(),
            Expression::BinaryOp { left, op, right } => format!(
                "({} {} {})",
                self.expression(left, inline),
                match op {
                    BinaryOperator::Add => "+",
                    BinaryOperator::Subtract => "-",
                    BinaryOperator::Multiply => "*",
                    BinaryOperator::Divide => "/",
                    BinaryOperator::Modulo => "%",
                    BinaryOperator::Concat => "||",
                },
                self.expression(right, inline),
            ),
            Expression::Function { name, args } => format!(
                "{name}({})",
                args.iter()
                    .map(|a| self.expression(a, inline))
                    .collect::<Vec<_>>()
                    .join(", "),
            ),
            Expression::Aggregate(agg) => self.aggregate(agg),
            Expression::Case { operand, when_then, else_result } => format!(
                "CASE{} {} {} END",
                operand
                    .as_ref()
                    .map(|e| format!(" {}", self.expression(e, inline)))
                    .unwrap_or_default(),
                when_then
                    .iter()
                    .map(|(when, then)| format!(
                        "WHEN {} THEN {}",
                        self.expression(when, inline),
                        self.expression(then, inline),
                    ))
                    .collect::<Vec<_>>()
                    .join(" "),
                else_result
                    .as_ref()
                    .map(|e| format!("ELSE {}", self.expression(e, inline)))
                    .unwrap_or_default(),
            ),
        }
    }

    fn aggregate(&self, agg: &AggregateFunction) -> String {
        match agg {
            AggregateFunction::Count(CountArg::All) => "count(*)".to_string(),
            AggregateFunction::Count(CountArg::Column(col)) => format!("count({col})"),
            AggregateFunction::Count(CountArg::Distinct(col)) => format!("count(DISTINCT {col})"),
            AggregateFunction::Sum(col) => format!("sum({col})"),
            AggregateFunction::Avg(col) => format!("avg({col})"),
            AggregateFunction::Min(col) => format!("min({col})"),
            AggregateFunction::Max(col) => format!("max({col})"),
            AggregateFunction::ApproxCountDistinct(col) => format!("uniqHLL12({col})"),
            AggregateFunction::Uniq(col) => format!("uniq({col})"),
            AggregateFunction::Quantile { level, column } => format!("quantile({level})({column})"),
            AggregateFunction::TopK { k, column } => format!("topK({k})({column})"),
            AggregateFunction::Histogram { bins, column } => format!("histogram({bins})({column})"),
            AggregateFunction::State(inner) => {
                let base = self.aggregate(inner);
                let paren = base.find('(').unwrap_or(base.len());
                format!("{}State{}", &base[..paren], &base[paren..])
            }
            AggregateFunction::Merge(inner) => {
                let base = self.aggregate(inner);
                let paren = base.find('(').unwrap_or(base.len());
                format!("{}Merge{}", &base[..paren], &base[paren..])
            }
        }
    }

    pub fn transpile(&mut self, statement: &Statement) -> Result<String, Error> {
        match statement {
            Statement::Select(s) => self.select(s, false),
            Statement::Insert(s) => self.insert(s),
            Statement::Update(s) => self.update(s),
            Statement::Delete(s) => self.delete(s),
            Statement::CreateTable(s) => self.create_table(s),
            Statement::AlterTable(s) => self.alter_table(s),
            Statement::DropTable(s) => self.drop_table(s),
            Statement::CreateView(s) => self.create_view(s),
            Statement::CreateMaterializedView(s) => self.create_materialized_view(s),
            Statement::DropView(s) => self.drop_view(s),
            Statement::Raw(sql, params) => {
                self.params.extend(params.iter().cloned());
                Ok(sql.clone())
            }
        }
    }

    pub fn finish(self, sql: String) -> PreparedStatement {
        PreparedStatement {
            sql,
            params: self.params,
        }
    }

    pub fn condition(&mut self, expr: &ConditionExpression, inline: bool) -> String {
        match expr {
            ConditionExpression::Eq(col, val) => format!("{col} = {}", self.placeholder(val.clone(), inline)),
            ConditionExpression::Ne(col, val) => format!("{col} != {}", self.placeholder(val.clone(), inline)),
            ConditionExpression::Gt(col, val) => format!("{col} > {}", self.placeholder(val.clone(), inline)),
            ConditionExpression::Gte(col, val) => format!("{col} >= {}", self.placeholder(val.clone(), inline)),
            ConditionExpression::Lt(col, val) => format!("{col} < {}", self.placeholder(val.clone(), inline)),
            ConditionExpression::Lte(col, val) => format!("{col} <= {}", self.placeholder(val.clone(), inline)),
            ConditionExpression::In(col, vals) => format!(
                "{col} IN ({})",
                vals.iter().map(|v| self.placeholder(v.clone(), inline)).collect::<Vec<_>>().join(", "),
            ),
            ConditionExpression::NotIn(col, vals) => format!(
                "{col} NOT IN ({})",
                vals.iter().map(|v| self.placeholder(v.clone(), inline)).collect::<Vec<_>>().join(", "),
            ),
            ConditionExpression::IsNull(col) => format!("{col} IS NULL"),
            ConditionExpression::IsNotNull(col) => format!("{col} IS NOT NULL"),
            ConditionExpression::Like(col, pattern) => {
                format!("{col} LIKE {}", self.placeholder(Value::String(pattern.clone()), inline))
            }
            ConditionExpression::NotLike(col, pattern) => {
                format!("{col} NOT LIKE {}", self.placeholder(Value::String(pattern.clone()), inline))
            }
            ConditionExpression::Between(col, low, high) => format!(
                "{col} BETWEEN {} AND {}",
                self.placeholder(low.clone(), inline),
                self.placeholder(high.clone(), inline),
            ),
            ConditionExpression::NotBetween(col, low, high) => format!(
                "{col} NOT BETWEEN {} AND {}",
                self.placeholder(low.clone(), inline),
                self.placeholder(high.clone(), inline),
            ),
            ConditionExpression::And(left, right) => {
                format!("({} AND {})", self.condition(left, inline), self.condition(right, inline))
            }
            ConditionExpression::Or(left, right) => {
                format!("({} OR {})", self.condition(left, inline), self.condition(right, inline))
            }
            ConditionExpression::Not(inner) => format!("NOT ({})", self.condition(inner, inline)),
        }
    }
}

#[cfg(test)]
mod tests {
    use pillar_core::{
        ast::{
            AggregateFunction, AlterTableStatement, ColumnDefinition, CountArg,
            CreateMaterializedViewStatement, CreateTableStatement, CreateViewStatement,
            DeleteStatement, DropTableStatement, DropViewStatement, InsertStatement, Interval,
            IntervalUnit, Projection, SelectStatement, Statement, TableRef, TtlClause,
            UpdateStatement, AggregateFn, AggregateStateFunction, ColumnType
        },
        condition::ConditionExpression,
        value::Value,
    };

    use super::Transpiler;

    fn transpile(statement: Statement) -> (String, Vec<Value>) {
        let mut t = Transpiler::new();
        let sql = t.transpile(&statement).unwrap();
        let prepared = t.finish(sql);
        (prepared.sql, prepared.params)
    }

    fn col(name: &str, data_type: ColumnType) -> ColumnDefinition {
        ColumnDefinition::new(name, data_type)
    }

    fn col_nullable(name: &str, data_type: ColumnType) -> ColumnDefinition {
        ColumnDefinition::new(name, data_type).nullable()
    }

    #[test]
    fn test_select_all() {
        let (sql, params) = transpile(Statement::Select(SelectStatement::new(TableRef::new("events"))));
        assert_eq!(sql, "SELECT * FROM events");
        assert!(params.is_empty());
    }

    #[test]
    fn test_select_with_filter() {
        let (sql, params) = transpile(Statement::Select(
            SelectStatement::new(TableRef::new("events"))
                .where_clause(ConditionExpression::eq("id", 42i64)),
        ));
        assert_eq!(sql, "SELECT * FROM events WHERE id = ?");
        assert_eq!(params, vec![Value::Int64(42)]);
    }

    #[test]
    fn test_select_limit_offset() {
        let (sql, params) = transpile(Statement::Select(
            SelectStatement::new(TableRef::new("events")).limit(10).offset(20),
        ));
        assert_eq!(sql, "SELECT * FROM events LIMIT 10 OFFSET 20");
        assert!(params.is_empty());
    }

    #[test]
    fn test_select_approx_count_distinct() {
        let (sql, params) = transpile(Statement::Select(
            SelectStatement::new(TableRef::new("events"))
                .projections(vec![Projection::Aggregate(AggregateFunction::ApproxCountDistinct("user_id".into()))]),
        ));
        assert_eq!(sql, "SELECT uniqHLL12(user_id) FROM events");
        assert!(params.is_empty());
    }

    #[test]
    fn test_select_aggregate_state_and_merge() {
        let (sql, params) = transpile(Statement::Select(
            SelectStatement::new(TableRef::new("events"))
                .projections(vec![
                    Projection::Aggregate(AggregateFunction::State(Box::new(AggregateFunction::Count(CountArg::All)))),
                    Projection::Aggregate(AggregateFunction::State(Box::new(AggregateFunction::Avg("duration_ms".into())))),
                    Projection::Aggregate(AggregateFunction::State(Box::new(AggregateFunction::Quantile { level: 0.95, column: "duration_ms".into() }))),
                ]),
        ));
        assert_eq!(sql, "SELECT countState(*), avgState(duration_ms), quantileState(0.95)(duration_ms) FROM events");
        assert!(params.is_empty());
    }

    #[test]
    fn test_select_aggregate_merge() {
        let (sql, params) = transpile(Statement::Select(
            SelectStatement::new(TableRef::new("events_by_minute"))
                .projections(vec![
                    Projection::Aggregate(AggregateFunction::Merge(Box::new(AggregateFunction::Count(CountArg::All)))),
                    Projection::Aggregate(AggregateFunction::Merge(Box::new(AggregateFunction::Quantile { level: 0.95, column: "duration_ms".into() }))),
                ]),
        ));
        assert_eq!(sql, "SELECT countMerge(*), quantileMerge(0.95)(duration_ms) FROM events_by_minute");
        assert!(params.is_empty());
    }

    #[test]
    fn test_select_quantile() {
        let (sql, params) = transpile(Statement::Select(
            SelectStatement::new(TableRef::new("events"))
                .projections(vec![Projection::Aggregate(AggregateFunction::Quantile { level: 0.99, column: "latency".into() })]),
        ));
        assert_eq!(sql, "SELECT quantile(0.99)(latency) FROM events");
        assert!(params.is_empty());
    }

    #[test]
    fn test_insert_single_row() {
        let (sql, params) = transpile(Statement::Insert(
            InsertStatement::new(TableRef::new("events"))
                .columns(vec!["id".into(), "name".into()])
                .values(vec![vec![Value::Int64(1), Value::String("foo".into())]]),
        ));
        assert_eq!(sql, "INSERT INTO events (id, name) VALUES (?, ?)");
        assert_eq!(params, vec![Value::Int64(1), Value::String("foo".into())]);
    }

    #[test]
    fn test_insert_multiple_rows() {
        let (sql, params) = transpile(Statement::Insert(
            InsertStatement::new(TableRef::new("events"))
                .columns(vec!["id".into(), "name".into()])
                .values(vec![
                    vec![Value::Int64(1), Value::String("a".into())],
                    vec![Value::Int64(2), Value::String("b".into())],
                ]),
        ));
        assert_eq!(sql, "INSERT INTO events (id, name) VALUES (?, ?), (?, ?)");
        assert_eq!(params, vec![
            Value::Int64(1),
            Value::String("a".into()),
            Value::Int64(2),
            Value::String("b".into()),
        ]);
    }

    #[test]
    fn test_update_emits_alter_table() {
        let (sql, params) = transpile(Statement::Update(
            UpdateStatement::new(TableRef::new("events"))
                .set(vec![("count".into(), Value::Int32(99))])
                .where_clause(ConditionExpression::eq("id", 1i64)),
        ));
        assert_eq!(sql, "ALTER TABLE events UPDATE count = ? WHERE id = ?");
        assert_eq!(params, vec![Value::Int32(99), Value::Int64(1)]);
    }

    #[test]
    fn test_delete_emits_alter_table() {
        let (sql, params) = transpile(Statement::Delete(
            DeleteStatement::new(TableRef::new("events"))
                .where_clause(ConditionExpression::eq("id", 1i64)),
        ));
        assert_eq!(sql, "ALTER TABLE events DELETE WHERE id = ?");
        assert_eq!(params, vec![Value::Int64(1)]);
    }

    #[test]
    fn test_delete_all() {
        let (sql, params) = transpile(Statement::Delete(DeleteStatement::new(TableRef::new("events"))));
        assert_eq!(sql, "ALTER TABLE events DELETE");
        assert!(params.is_empty());
    }

    #[test]
    fn test_create_table_with_engine_options() {
        let (sql, params) = transpile(Statement::CreateTable(
            CreateTableStatement::new("events")
                .columns(vec![col("id", ColumnType::UInt64)])
                .option("engine", "MergeTree()")
                .option("order_by", "id"),
        ));
        assert_eq!(sql, "CREATE TABLE events (id UInt64) ENGINE = MergeTree() ORDER BY id");
        assert!(params.is_empty());
    }

    #[test]
    fn test_create_table_low_cardinality_and_datetime64() {
        let (sql, params) = transpile(Statement::CreateTable(
            CreateTableStatement::new("events")
                .columns(vec![
                    col("event_time", ColumnType::DateTime64 { precision: 3 }),
                    col("event_type", ColumnType::LowCardinalityString),
                ]),
        ));
        assert_eq!(sql, "CREATE TABLE events (event_time DateTime64(3), event_type LowCardinality(String))");
        assert!(params.is_empty());
    }

    #[test]
    fn test_create_table_aggregate_state_column() {
        let (sql, params) = transpile(Statement::CreateTable(
            CreateTableStatement::new("events_by_minute")
                .columns(vec![
                    col("count", ColumnType::AggregateState(AggregateStateFunction::new(AggregateFn::Count, vec![ColumnType::UInt64]))),
                    col("duration_ms_avg", ColumnType::AggregateState(AggregateStateFunction::new(AggregateFn::Avg, vec![ColumnType::UInt32]))),
                    col("duration_ms_p95", ColumnType::AggregateState(AggregateStateFunction::new(AggregateFn::Quantile(0.95), vec![ColumnType::UInt32]))),
                ]),
        ));
        assert_eq!(
            sql,
            "CREATE TABLE events_by_minute (count AggregateFunction(count, UInt64), duration_ms_avg AggregateFunction(avg, UInt32), duration_ms_p95 AggregateFunction(quantile(0.95), UInt32))",
        );
        assert!(params.is_empty());
    }

    #[test]
    fn test_create_table_if_not_exists_nullable() {
        let (sql, params) = transpile(Statement::CreateTable(
            CreateTableStatement::new("events")
                .if_not_exists()
                .columns(vec![col_nullable("ts", ColumnType::DateTime64 { precision: 3 })]),
        ));
        assert_eq!(sql, "CREATE TABLE IF NOT EXISTS events (ts Nullable(DateTime64(3)))");
        assert!(params.is_empty());
    }

    #[test]
    fn test_create_table_with_ttl() {
        let (sql, params) = transpile(Statement::CreateTable(
            CreateTableStatement::new("events")
                .columns(vec![col("event_time", ColumnType::DateTime64 { precision: 3 })])
                .option("engine", "MergeTree()")
                .option("order_by", "event_time")
                .ttl(TtlClause::delete("event_time", Interval::new(90, IntervalUnit::Day))),
        ));
        assert_eq!(sql, "CREATE TABLE events (event_time DateTime64(3)) ENGINE = MergeTree() ORDER BY event_time TTL event_time + INTERVAL 90 DAY");
        assert!(params.is_empty());
    }

    #[test]
    fn test_alter_table_add_drop_columns() {
        let (sql, params) = transpile(Statement::AlterTable(
            AlterTableStatement::new("events")
                .add_columns(vec![col("severity", ColumnType::UInt8)])
                .drop_columns(vec!["old_col".into()]),
        ));
        assert_eq!(sql, "ALTER TABLE events ADD COLUMN severity UInt8, DROP COLUMN old_col");
        assert!(params.is_empty());
    }

    #[test]
    fn test_alter_table_modify_ttl() {
        let (sql, params) = transpile(Statement::AlterTable(
            AlterTableStatement::new("events")
                .ttl(TtlClause::delete("event_time", Interval::new(30, IntervalUnit::Day))),
        ));
        assert_eq!(sql, "ALTER TABLE events  MODIFY TTL event_time + INTERVAL 30 DAY");
        assert!(params.is_empty());
    }

    #[test]
    fn test_drop_table_if_exists() {
        let (sql, params) = transpile(Statement::DropTable(
            DropTableStatement::new("events").if_exists(),
        ));
        assert_eq!(sql, "DROP TABLE IF EXISTS events");
        assert!(params.is_empty());
    }

    #[test]
    fn test_create_view_inlines_values() {
        let (sql, params) = transpile(Statement::CreateView(
            CreateViewStatement::new(
                "high_severity",
                SelectStatement::new(TableRef::new("events"))
                    .where_clause(ConditionExpression::gte("severity", 3i32)),
            ),
        ));
        assert_eq!(sql, "CREATE VIEW high_severity AS SELECT * FROM events WHERE severity >= 3");
        assert!(params.is_empty());
    }

    #[test]
    fn test_create_view_or_replace() {
        let (sql, params) = transpile(Statement::CreateView(
            CreateViewStatement::new(
                "high_severity",
                SelectStatement::new(TableRef::new("events")),
            )
            .or_replace(),
        ));
        assert_eq!(sql, "CREATE OR REPLACE VIEW high_severity AS SELECT * FROM events");
        assert!(params.is_empty());
    }

    #[test]
    fn test_create_materialized_view_to_table() {
        let (sql, params) = transpile(Statement::CreateMaterializedView(
            CreateMaterializedViewStatement::new(
                "high_severity_mv",
                SelectStatement::new(TableRef::new("events"))
                    .where_clause(ConditionExpression::gte("severity", 3i32)),
            )
            .to_table("high_severity_local")
            .option("engine", "MergeTree()")
            .option("populate", "true"),
        ));
        assert_eq!(
            sql,
            "CREATE MATERIALIZED VIEW high_severity_mv TO high_severity_local ENGINE = MergeTree() POPULATE AS SELECT * FROM events WHERE severity >= 3",
        );
        assert!(params.is_empty());
    }

    #[test]
    fn test_drop_view() {
        let (sql, params) = transpile(Statement::DropView(
            DropViewStatement::new("high_severity").if_exists(),
        ));
        assert_eq!(sql, "DROP VIEW IF EXISTS high_severity");
        assert!(params.is_empty());
    }

    #[test]
    fn test_drop_materialized_view_emits_drop_table() {
        let (sql, params) = transpile(Statement::DropView(
            DropViewStatement::new("high_severity_mv").materialized(),
        ));
        assert_eq!(sql, "DROP TABLE high_severity_mv");
        assert!(params.is_empty());
    }

    #[test]
    fn test_condition_and() {
        let (sql, params) = transpile(Statement::Select(
            SelectStatement::new(TableRef::new("events")).where_clause(
                ConditionExpression::eq("id", 1i64).and(ConditionExpression::eq("count", 6i32)),
            ),
        ));
        assert_eq!(sql, "SELECT * FROM events WHERE (id = ? AND count = ?)");
        assert_eq!(params, vec![Value::Int64(1), Value::Int32(6)]);
    }

    #[test]
    fn test_insert_empty_returns_error() {
        let mut t = Transpiler::new();
        let result = t.transpile(&Statement::Insert(
            InsertStatement::new(TableRef::new("events"))
                .columns(vec!["id".into()])
                .values(vec![]),
        ));
        assert!(result.is_err());
    }

    #[test]
    fn test_raw_passthrough() {
        let (sql, params) = transpile(Statement::Raw(
            "SELECT 1".into(),
            vec![Value::Int32(1)],
        ));
        assert_eq!(sql, "SELECT 1");
        assert_eq!(params, vec![Value::Int32(1)]);
    }
}
