use pillar_core::{
    ast::{
        AggregateFunction, AlterTableStatement, BinaryOperator, ColumnDefinition, CountArg,
        CreateMaterializedViewStatement, CreateTableStatement, CreateViewStatement,
        DeleteStatement, DropTableStatement, DropViewStatement, Expression, InsertStatement,
        JoinType, NullsOrder, OnConflictAction, OrderDirection, Projection, SelectStatement,
        Statement, UpdateStatement,
    },
    condition::ConditionExpression,
    errors::Error,
    dialect::PreparedStatement,
    value::{ToSql, Value},
};


pub(crate) struct Transpiler {
    params: Vec<Value>,
    count: usize,
}

impl Transpiler {
    pub(crate) fn new() -> Self {
        Self { params: Vec::new(), count: 0 }
    }

    fn placeholder(&mut self, value: Value, inline: bool) -> String {
        if inline {
            return value.to_sql();
        }
        self.params.push(value);
        self.count += 1;
        format!("${}", self.count)
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

        let mut sql = format!(
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
        );

        if let Some(on_conflict) = &stmt.on_conflict {
            let targets = on_conflict.target.join(", ");

            match &on_conflict.action {
                OnConflictAction::DoNothing => {
                    sql.push_str(&format!(" ON CONFLICT ({targets}) DO NOTHING"));
                }

                OnConflictAction::DoUpdate { set, where_clause } => {
                    sql.push_str(&format!(
                        " ON CONFLICT ({targets}) DO UPDATE SET {}",
                        set.iter()
                            .map(|(col, val)| format!("{col} = {}", self.placeholder(val.clone(), false)))
                            .collect::<Vec<_>>()
                            .join(", "),
                    ));

                    if let Some(cond) = where_clause {
                        sql.push_str(&format!(" WHERE {}", self.condition(cond, false)));
                    }
                }
            }
        }

        Ok(sql)
    }

    fn update(&mut self, stmt: &UpdateStatement) -> Result<String, Error> {
        if stmt.set.is_empty() {
            return Err(Error::invalid_query("UPDATE statement has no SET clauses"));
        }

        let mut sql = format!(
            "UPDATE {} SET {}",
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
        let mut sql = format!("DELETE FROM {}", stmt.table.name);

        if let Some(where_clause) = &stmt.where_clause {
            sql.push_str(&format!(" WHERE {}", self.condition(where_clause, false)));
        }

        Ok(sql)
    }

    fn create_table(&mut self, stmt: &CreateTableStatement) -> Result<String, Error> {
        Ok(format!(
            "CREATE TABLE{} {} ({})",
            if stmt.if_not_exists { " IF NOT EXISTS" } else { "" },
            stmt.name,
            stmt.columns
                .iter()
                .map(|col| self.column_definition(col))
                .collect::<Vec<_>>()
                .join(", "),
        ))
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

        if parts.is_empty() {
            return Err(Error::invalid_query("ALTER TABLE statement has no operations"));
        }

        Ok(format!("ALTER TABLE {} {}", stmt.name, parts.join(", ")))
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
        // DuckDB has no materialized views, so we emit a plain view
        Ok(format!(
            "CREATE{} VIEW{} {} AS {}",
            if stmt.or_replace { " OR REPLACE" } else { "" },
            if stmt.if_not_exists { " IF NOT EXISTS" } else { "" },
            stmt.name,
            self.select(&stmt.query, true)?,
        ))
    }

    fn drop_view(&mut self, stmt: &DropViewStatement) -> Result<String, Error> {
        Ok(format!(
            "DROP VIEW{} {}",
            if stmt.if_exists { " IF EXISTS" } else { "" },
            stmt.name,
        ))
    }

    fn column_definition(&self, col: &ColumnDefinition) -> String {
        format!(
            "{} {}{}{}{}",
            col.name,
            col.data_type,
            if col.nullable { "" } else { " NOT NULL" },
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

            Projection::Aggregate(agg) => match agg {
                AggregateFunction::Count(CountArg::All) => "COUNT(*)".to_string(),
                AggregateFunction::Count(CountArg::Column(col)) => format!("COUNT({col})"),
                AggregateFunction::Count(CountArg::Distinct(col)) => {
                    format!("COUNT(DISTINCT {col})")
                }
                AggregateFunction::Sum(col) => format!("SUM({col})"),
                AggregateFunction::Avg(col) => format!("AVG({col})"),
                AggregateFunction::Min(col) => format!("MIN({col})"),
                AggregateFunction::Max(col) => format!("MAX({col})"),
                AggregateFunction::ApproxCountDistinct(col) => {
                    format!("APPROX_COUNT_DISTINCT({col})")
                }
            },

            Projection::Expression(expr) => self.expression(expr, inline),
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

    pub(crate) fn transpile(&mut self, statement: &Statement) -> Result<String, Error> {
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

    pub(crate) fn finish(self, sql: String) -> PreparedStatement {
        PreparedStatement {
            sql,
            params: self.params,
        }
    }

    pub(crate) fn condition(&mut self, expr: &ConditionExpression, inline: bool) -> String {
        match expr {
            ConditionExpression::Eq(col, val) => {
                format!("{col} = {}", self.placeholder(val.clone(), inline))
            }

            ConditionExpression::Ne(col, val) => {
                format!("{col} != {}", self.placeholder(val.clone(), inline))
            }

            ConditionExpression::Gt(col, val) => {
                format!("{col} > {}", self.placeholder(val.clone(), inline))
            }

            ConditionExpression::Gte(col, val) => {
                format!("{col} >= {}", self.placeholder(val.clone(), inline))
            }

            ConditionExpression::Lt(col, val) => {
                format!("{col} < {}", self.placeholder(val.clone(), inline))
            }

            ConditionExpression::Lte(col, val) => {
                format!("{col} <= {}", self.placeholder(val.clone(), inline))
            }

            ConditionExpression::In(col, vals) => format!(
                "{col} IN ({})",
                vals.iter()
                    .map(|v| self.placeholder(v.clone(), inline))
                    .collect::<Vec<_>>()
                    .join(", "),
            ),

            ConditionExpression::NotIn(col, vals) => format!(
                "{col} NOT IN ({})",
                vals.iter()
                    .map(|v| self.placeholder(v.clone(), inline))
                    .collect::<Vec<_>>()
                    .join(", "),
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
            DeleteStatement, InsertStatement, SelectStatement, Statement, TableRef, UpdateStatement,
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

    #[test]
    fn test_select_all() {
        let (sql, params) = transpile(Statement::Select(SelectStatement::new(
            TableRef::new("events"),
        )));

        assert_eq!(sql, "SELECT * FROM events");
        assert!(params.is_empty());
    }

    #[test]
    fn test_select_with_filter() {
        let (sql, params) = transpile(Statement::Select(
            SelectStatement::new(TableRef::new("events"))
                .where_clause(ConditionExpression::eq("id", 42i64)),
        ));

        assert_eq!(sql, "SELECT * FROM events WHERE id = $1");
        assert_eq!(params, vec![Value::Int64(42)]);
    }

    #[test]
    fn test_select_limit_offset() {
        let (sql, params) = transpile(Statement::Select(
            SelectStatement::new(TableRef::new("events"))
                .limit(10)
                .offset(20),
        ));

        assert_eq!(sql, "SELECT * FROM events LIMIT 10 OFFSET 20");
        assert!(params.is_empty());
    }

    #[test]
    fn test_insert_single_row() {
        let (sql, params) = transpile(Statement::Insert(
            InsertStatement::new(TableRef::new("events"))
                .columns(vec!["id".into(), "name".into(), "count".into()])
                .values(vec![vec![
                    Value::Int64(1),
                    Value::String("foo".into()),
                    Value::Int32(5),
                ]]),
        ));

        assert_eq!(sql, "INSERT INTO events (id, name, count) VALUES ($1, $2, $3)");
        assert_eq!(params, vec![
            Value::Int64(1),
            Value::String("foo".into()),
            Value::Int32(5),
        ]);
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

        assert_eq!(sql, "INSERT INTO events (id, name) VALUES ($1, $2), ($3, $4)");
        assert_eq!(params, vec![
            Value::Int64(1),
            Value::String("a".into()),
            Value::Int64(2),
            Value::String("b".into()),
        ]);
    }

    #[test]
    fn test_update_with_filter() {
        let (sql, params) = transpile(Statement::Update(
            UpdateStatement::new(TableRef::new("events"))
                .set(vec![("count".into(), Value::Int32(99))])
                .where_clause(ConditionExpression::eq("id", 1i64)),
        ));

        assert_eq!(sql, "UPDATE events SET count = $1 WHERE id = $2");
        assert_eq!(params, vec![Value::Int32(99), Value::Int64(1)]);
    }

    #[test]
    fn test_delete_with_filter() {
        let (sql, params) = transpile(Statement::Delete(
            DeleteStatement::new(TableRef::new("events"))
                .where_clause(ConditionExpression::eq("id", 1i64)),
        ));

        assert_eq!(sql, "DELETE FROM events WHERE id = $1");
        assert_eq!(params, vec![Value::Int64(1)]);
    }

    #[test]
    fn test_delete_all() {
        let (sql, params) = transpile(Statement::Delete(DeleteStatement::new(TableRef::new(
            "events",
        ))));

        assert_eq!(sql, "DELETE FROM events");
        assert!(params.is_empty());
    }

    #[test]
    fn test_condition_and() {
        let (sql, params) = transpile(Statement::Select(
            SelectStatement::new(TableRef::new("events")).where_clause(
                ConditionExpression::eq("id", 1i64)
                    .and(ConditionExpression::eq("count", 6i32)),
            ),
        ));

        assert_eq!(sql, "SELECT * FROM events WHERE (id = $1 AND count = $2)");
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
}
