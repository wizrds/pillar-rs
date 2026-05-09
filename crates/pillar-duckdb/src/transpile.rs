use pillar_core::{
    ast::{
        AggregateFunction, AlterTableStatement, BinaryOperator, ColumnDefinition, ColumnType,
        CompoundSelect, CountArg, Cte, CreateMaterializedViewStatement, CreateTableStatement,
        CreateViewStatement, DeleteStatement, DropTableStatement, DropViewStatement, Expression,
        FrameBound, FrameUnit, FromSource, InsertBody, InsertStatement, JoinType, NullsOrder,
        OnConflictAction, OrderDirection, Projection, SelectStatement, SetOperator, Statement,
        UpdateStatement, AggregateFn, WindowFunc, WindowFunction, WindowSpec,
    },
    condition::{ComparisonOp, ConditionExpression},
    errors::Error,
    dialect::PreparedStatement,
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

    fn value_to_sql(&self, value: &Value) -> String {
        value.to_sql()
    }

    fn placeholder(&mut self, value: Value, inline: bool) -> String {
        if inline {
            return self.value_to_sql(&value);
        }
        self.params.push(value);
        self.count += 1;
        format!("${}", self.count)
    }

    fn from_source(&mut self, src: &FromSource, inline: bool) -> Result<String, Error> {
        match src {
            FromSource::Table(t) => Ok(match &t.alias {
                Some(alias) => format!("{} AS {}", t.name, alias),
                None => t.name.clone(),
            }),
            FromSource::Subquery { query, alias } => {
                let inner = self.select(query, inline)?;
                Ok(format!("({inner}) AS {alias}"))
            }
        }
    }

    fn cte_clause(&mut self, ctes: &[Cte], inline: bool) -> Result<String, Error> {
        if ctes.is_empty() {
            return Ok(String::new());
        }

        let has_recursive = ctes.iter().any(|c| c.recursive);
        let prefix = if has_recursive { "WITH RECURSIVE " } else { "WITH " };

        let rendered = ctes
            .iter()
            .map(|cte| {
                let cols = if cte.columns.is_empty() {
                    String::new()
                } else {
                    format!(
                        " ({})",
                        cte.columns.iter().map(|c| c.name.as_str()).collect::<Vec<_>>().join(", "),
                    )
                };
                let query = self.select(&cte.query, inline)?;
                Ok(format!("{}{cols} AS ({query})", cte.name))
            })
            .collect::<Result<Vec<_>, Error>>()?;

        Ok(format!("{prefix}{} ", rendered.join(", ")))
    }

    fn select(&mut self, stmt: &SelectStatement, inline: bool) -> Result<String, Error> {
        let with = self.cte_clause(&stmt.with, inline)?;
        let from = self.from_source(&stmt.from, inline)?;

        let mut sql = format!(
            "{}{} {} FROM {}",
            with,
            if stmt.distinct { "SELECT DISTINCT" } else { "SELECT" },
            stmt.projections
                .iter()
                .map(|p| self.projection(p, inline))
                .collect::<Vec<_>>()
                .join(", "),
            from,
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
            sql.push_str(&format!(
                " GROUP BY {}",
                stmt.group_by.iter().map(|c| c.name.as_str()).collect::<Vec<_>>().join(", "),
            ));
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
                        o.column.name,
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
        let cols = stmt.columns.iter().map(|c| c.name.as_str()).collect::<Vec<_>>().join(", ");

        let body = match &stmt.body {
            InsertBody::Values(rows) => {
                if rows.is_empty() {
                    return Err(Error::invalid_query("INSERT statement has no rows"));
                }

                let col_count = stmt.columns.len();

                for (i, row) in rows.iter().enumerate() {
                    if row.len() != col_count {
                        return Err(Error::invalid_query(format!(
                            "row {i} has {} values but {col_count} columns were specified",
                            row.len(),
                        )));
                    }
                }

                format!(
                    "VALUES {}",
                    rows.iter()
                        .map(|row| format!(
                            "({})",
                            row.iter()
                                .map(|v| self.placeholder(v.clone(), false))
                                .collect::<Vec<_>>()
                                .join(", "),
                        ))
                        .collect::<Vec<_>>()
                        .join(", "),
                )
            }

            InsertBody::Select(query) => self.select(query, false)?,
        };

        let mut sql = if cols.is_empty() {
            format!("INSERT INTO {} {body}", stmt.table.name)
        } else {
            format!("INSERT INTO {} ({cols}) {body}", stmt.table.name)
        };

        if let Some(on_conflict) = &stmt.on_conflict {
            let targets = on_conflict.target.iter().map(|c| c.name.as_str()).collect::<Vec<_>>().join(", ");

            match &on_conflict.action {
                OnConflictAction::DoNothing => {
                    sql.push_str(&format!(" ON CONFLICT ({targets}) DO NOTHING"));
                }

                OnConflictAction::DoUpdate { set, where_clause } => {
                    sql.push_str(&format!(
                        " ON CONFLICT ({targets}) DO UPDATE SET {}",
                        set.iter()
                            .map(|(col, val)| format!("{} = {}", col.name, self.placeholder(val.clone(), false)))
                            .collect::<Vec<_>>()
                            .join(", "),
                    ));

                    if let Some(cond) = where_clause {
                        sql.push_str(&format!(" WHERE {}", self.condition(cond, false)));
                    }
                }
            }
        }

        if let Some(returning) = &stmt.returning {
            sql.push_str(&format!(
                " RETURNING {}",
                returning.iter().map(|p| self.projection(p, false)).collect::<Vec<_>>().join(", "),
            ));
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
                .map(|(col, val)| format!("{} = {}", col.name, self.placeholder(val.clone(), false)))
                .collect::<Vec<_>>()
                .join(", "),
        );

        if let Some(where_clause) = &stmt.where_clause {
            sql.push_str(&format!(" WHERE {}", self.condition(where_clause, false)));
        }

        if let Some(returning) = &stmt.returning {
            sql.push_str(&format!(
                " RETURNING {}",
                returning.iter().map(|p| self.projection(p, false)).collect::<Vec<_>>().join(", "),
            ));
        }

        Ok(sql)
    }

    fn delete(&mut self, stmt: &DeleteStatement) -> Result<String, Error> {
        let mut sql = format!("DELETE FROM {}", stmt.table.name);

        if let Some(where_clause) = &stmt.where_clause {
            sql.push_str(&format!(" WHERE {}", self.condition(where_clause, false)));
        }

        if let Some(returning) = &stmt.returning {
            sql.push_str(&format!(
                " RETURNING {}",
                returning.iter().map(|p| self.projection(p, false)).collect::<Vec<_>>().join(", "),
            ));
        }

        Ok(sql)
    }

    fn compound(&mut self, stmt: &CompoundSelect) -> Result<String, Error> {
        let left = self.select(&stmt.left, false)?;
        let right = self.select(&stmt.right, false)?;
        let op = match stmt.operator {
            SetOperator::UnionAll => "UNION ALL",
            SetOperator::Union => "UNION",
            SetOperator::Intersect => "INTERSECT",
            SetOperator::Except => "EXCEPT",
        };
        Ok(format!("({left}) {op} ({right})"))
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
                    .map(|col| format!("DROP COLUMN {}", col.name)),
            )
            .collect::<Vec<_>>();

        if parts.is_empty() && stmt.ttl.is_none() {
            return Err(Error::invalid_query("ALTER TABLE statement has no operations"));
        }

        if parts.is_empty() {
            return Ok(String::new());
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
        // When a TO target is specified, the target table becomes the view on DuckDB.
        // We drop the pre-created table and replace it with a view of the same name,
        // so downstream reads from the target name work on both backends.
        let (view_name, preamble) = match &stmt.to_table {
            Some(target) => (
                target.as_str(),
                format!("DROP TABLE IF EXISTS {target}; "),
            ),
            None => (stmt.name.as_str(), String::new()),
        };

        Ok(format!(
            "{}CREATE{} VIEW{} {} AS {}",
            preamble,
            if stmt.or_replace { " OR REPLACE" } else { "" },
            if stmt.if_not_exists { " IF NOT EXISTS" } else { "" },
            view_name,
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

    fn column_type(&self, col_type: &ColumnType) -> String {
        match col_type {
            ColumnType::Boolean => "BOOLEAN".to_string(),
            ColumnType::Int8 => "TINYINT".to_string(),
            ColumnType::Int16 => "SMALLINT".to_string(),
            ColumnType::Int32 => "INTEGER".to_string(),
            ColumnType::Int64 => "BIGINT".to_string(),
            ColumnType::UInt8 => "UTINYINT".to_string(),
            ColumnType::UInt16 => "USMALLINT".to_string(),
            ColumnType::UInt32 => "UINTEGER".to_string(),
            ColumnType::UInt64 => "UBIGINT".to_string(),
            ColumnType::Float32 => "FLOAT".to_string(),
            ColumnType::Float64 => "DOUBLE".to_string(),
            ColumnType::String => "VARCHAR".to_string(),
            ColumnType::Binary => "BLOB".to_string(),
            ColumnType::List(inner) => format!("{}[]", self.column_type(inner)),
            ColumnType::Map(k, v) => format!("MAP({}, {})", self.column_type(k), self.column_type(v)),
            #[cfg(feature = "chrono")]
            ColumnType::Date => "DATE".to_string(),
            #[cfg(feature = "chrono")]
            ColumnType::Time => "TIME".to_string(),
            #[cfg(feature = "chrono")]
            ColumnType::DateTime => "TIMESTAMP WITH TIME ZONE".to_string(),
            #[cfg(feature = "uuid")]
            ColumnType::Uuid => "UUID".to_string(),
            ColumnType::DateTime64 { .. } => "TIMESTAMP".to_string(),
            ColumnType::LowCardinalityString => "VARCHAR".to_string(),
            ColumnType::FixedString(n) => format!("CHAR({n})"),
            ColumnType::AggregateState(state) => {
                // DuckDB has no aggregate state storage; use the result type of the function
                match &state.function {
                    AggregateFn::Count => "UBIGINT".to_string(),
                    AggregateFn::Sum | AggregateFn::Avg => state.arg_types.first()
                        .map(|t| self.column_type(t))
                        .unwrap_or_else(|| "DOUBLE".to_string()),
                    AggregateFn::Min | AggregateFn::Max => state.arg_types.first()
                        .map(|t| self.column_type(t))
                        .unwrap_or_else(|| "DOUBLE".to_string()),
                    AggregateFn::Uniq => "UBIGINT".to_string(),
                    AggregateFn::Quantile(_) => state.arg_types.first()
                        .map(|t| self.column_type(t))
                        .unwrap_or_else(|| "DOUBLE".to_string()),
                    AggregateFn::TopK(_) => state.arg_types.first()
                        .map(|t| format!("{}[]", self.column_type(t)))
                        .unwrap_or_else(|| "VARCHAR[]".to_string()),
                    AggregateFn::Histogram(_) => "DOUBLE[]".to_string(),
                    AggregateFn::Custom(_) => "VARCHAR".to_string(),
                }
            }
            ColumnType::Nullable(inner) => self.column_type(inner),
            ColumnType::Custom(s) => s.clone(),
        }
    }

    fn column_definition(&self, col: &ColumnDefinition) -> String {
        format!(
            "{} {}{}{}{}",
            col.name,
            self.column_type(&col.data_type),
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
            Projection::Column(col) => col.name.clone(),
            Projection::ColumnAlias(col, alias) => format!("{} AS {alias}", col.name),
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

            Expression::Column(col) => col.name.clone(),

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

            Expression::Aggregate(agg) => self.aggregate(agg),

            Expression::Cast { expr, to } => {
                format!("CAST({} AS {})", self.expression(expr, inline), self.column_type(to))
            }

            Expression::Window(wf) => self.window_function(wf, inline),

            Expression::Subquery(query) => {
                self.select(query, inline).unwrap_or_else(|_| "NULL".to_string())
            }
        }
    }

    fn window_function(&mut self, wf: &WindowFunction, inline: bool) -> String {
        format!("{} OVER ({})", self.window_func(&wf.func), self.window_spec(&wf.over, inline))
    }

    fn window_func(&self, func: &WindowFunc) -> String {
        match func {
            WindowFunc::RowNumber => "ROW_NUMBER()".to_string(),
            WindowFunc::Rank => "RANK()".to_string(),
            WindowFunc::DenseRank => "DENSE_RANK()".to_string(),
            WindowFunc::PercentRank => "PERCENT_RANK()".to_string(),
            WindowFunc::CumeDist => "CUME_DIST()".to_string(),
            WindowFunc::Ntile(n) => format!("NTILE({n})"),
            WindowFunc::FirstValue(col) => format!("FIRST_VALUE({})", col.name),
            WindowFunc::LastValue(col) => format!("LAST_VALUE({})", col.name),
            WindowFunc::NthValue { column, n } => format!("NTH_VALUE({}, {n})", column.name),
            WindowFunc::Lag { column, offset, default } => {
                match (offset, default) {
                    (None, None) => format!("LAG({})", column.name),
                    (Some(o), None) => format!("LAG({}, {o})", column.name),
                    (Some(o), Some(d)) => format!("LAG({}, {o}, {})", column.name, d.to_sql()),
                    (None, Some(d)) => format!("LAG({}, 1, {})", column.name, d.to_sql()),
                }
            }
            WindowFunc::Lead { column, offset, default } => {
                match (offset, default) {
                    (None, None) => format!("LEAD({})", column.name),
                    (Some(o), None) => format!("LEAD({}, {o})", column.name),
                    (Some(o), Some(d)) => format!("LEAD({}, {o}, {})", column.name, d.to_sql()),
                    (None, Some(d)) => format!("LEAD({}, 1, {})", column.name, d.to_sql()),
                }
            }
        }
    }

    fn window_spec(&mut self, spec: &WindowSpec, inline: bool) -> String {
        let mut parts = Vec::new();

        if !spec.partition_by.is_empty() {
            parts.push(format!(
                "PARTITION BY {}",
                spec.partition_by.iter().map(|c| c.name.as_str()).collect::<Vec<_>>().join(", "),
            ));
        }

        if !spec.order_by.is_empty() {
            parts.push(format!(
                "ORDER BY {}",
                spec.order_by
                    .iter()
                    .map(|o| format!(
                        "{} {}{}",
                        o.column.name,
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

        if let Some(frame) = &spec.frame {
            let unit = match frame.unit {
                FrameUnit::Rows => "ROWS",
                FrameUnit::Range => "RANGE",
                FrameUnit::Groups => "GROUPS",
            };
            let frame_sql = match &frame.end {
                None => format!("{unit} {}", self.frame_bound(&frame.start)),
                Some(end) => format!(
                    "{unit} BETWEEN {} AND {}",
                    self.frame_bound(&frame.start),
                    self.frame_bound(end),
                ),
            };
            parts.push(frame_sql);
        }

        let _ = inline;
        parts.join(" ")
    }

    fn frame_bound(&self, bound: &FrameBound) -> String {
        match bound {
            FrameBound::UnboundedPreceding => "UNBOUNDED PRECEDING".to_string(),
            FrameBound::Preceding(n) => format!("{n} PRECEDING"),
            FrameBound::CurrentRow => "CURRENT ROW".to_string(),
            FrameBound::Following(n) => format!("{n} FOLLOWING"),
            FrameBound::UnboundedFollowing => "UNBOUNDED FOLLOWING".to_string(),
        }
    }

    fn aggregate(&self, agg: &AggregateFunction) -> String {
        match agg {
            AggregateFunction::Count(CountArg::All) => "CAST(COUNT(*) AS UBIGINT)".to_string(),
            AggregateFunction::Count(CountArg::Column(col)) => format!("CAST(COUNT({}) AS UBIGINT)", col.name),
            AggregateFunction::Count(CountArg::Distinct(col)) => format!("CAST(COUNT(DISTINCT {}) AS UBIGINT)", col.name),
            AggregateFunction::Sum(col) => format!("CAST(SUM({}) AS UBIGINT)", col.name),
            AggregateFunction::Avg(col) => format!("AVG({})", col.name),
            AggregateFunction::Min(col) => format!("MIN({})", col.name),
            AggregateFunction::Max(col) => format!("MAX({})", col.name),
            AggregateFunction::ApproxCountDistinct(col) => format!("APPROX_COUNT_DISTINCT({})", col.name),
            AggregateFunction::Uniq(col) => format!("APPROX_COUNT_DISTINCT({})", col.name),
            AggregateFunction::Quantile { level, column } => {
                format!("PERCENTILE_CONT({level}) WITHIN GROUP (ORDER BY {})", column.name)
            }
            AggregateFunction::TopK { k, column } => format!("APPROX_TOP_K({}, {k})", column.name),
            AggregateFunction::Histogram { bins: _, column } => format!("histogram({})", column.name),
            // State writes plain aggregates; Merge reads the stored value directly
            AggregateFunction::State(inner) => self.aggregate(inner),
            AggregateFunction::Merge(inner) => self.merge_column(inner),
        }
    }

    fn merge_column(&self, inner: &AggregateFunction) -> String {
        format!("ANY_VALUE({})", match inner {
            AggregateFunction::Count(CountArg::Column(col)) => col.name.clone(),
            AggregateFunction::Count(CountArg::Distinct(col)) => col.name.clone(),
            AggregateFunction::Sum(col) => col.name.clone(),
            AggregateFunction::Avg(col) => col.name.clone(),
            AggregateFunction::Min(col) => col.name.clone(),
            AggregateFunction::Max(col) => col.name.clone(),
            AggregateFunction::Uniq(col) => col.name.clone(),
            AggregateFunction::ApproxCountDistinct(col) => col.name.clone(),
            AggregateFunction::Quantile { column, .. } => column.name.clone(),
            AggregateFunction::TopK { column, .. } => column.name.clone(),
            AggregateFunction::Histogram { column, .. } => column.name.clone(),
            _ => return self.aggregate(inner),
        })
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
            Statement::TableExists(name) => Ok(format!(
                "SELECT count(*) FROM information_schema.tables WHERE table_name = '{name}'"
            )),
            Statement::CreateView(s) => self.create_view(s),
            Statement::CreateMaterializedView(s) => self.create_materialized_view(s),
            Statement::DropView(s) => self.drop_view(s),
            Statement::Compound(s) => self.compound(s),
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
            ConditionExpression::Eq(col, val) => {
                format!("{} = {}", col.name, self.placeholder(val.clone(), inline))
            }

            ConditionExpression::Ne(col, val) => {
                format!("{} != {}", col.name, self.placeholder(val.clone(), inline))
            }

            ConditionExpression::Gt(col, val) => {
                format!("{} > {}", col.name, self.placeholder(val.clone(), inline))
            }

            ConditionExpression::Gte(col, val) => {
                format!("{} >= {}", col.name, self.placeholder(val.clone(), inline))
            }

            ConditionExpression::Lt(col, val) => {
                format!("{} < {}", col.name, self.placeholder(val.clone(), inline))
            }

            ConditionExpression::Lte(col, val) => {
                format!("{} <= {}", col.name, self.placeholder(val.clone(), inline))
            }

            ConditionExpression::In(col, vals) => format!(
                "{} IN ({})",
                col.name,
                vals.iter()
                    .map(|v| self.placeholder(v.clone(), inline))
                    .collect::<Vec<_>>()
                    .join(", "),
            ),

            ConditionExpression::NotIn(col, vals) => format!(
                "{} NOT IN ({})",
                col.name,
                vals.iter()
                    .map(|v| self.placeholder(v.clone(), inline))
                    .collect::<Vec<_>>()
                    .join(", "),
            ),

            ConditionExpression::IsNull(col) => format!("{} IS NULL", col.name),

            ConditionExpression::IsNotNull(col) => format!("{} IS NOT NULL", col.name),

            ConditionExpression::Like(col, pattern) => {
                format!("{} LIKE {}", col.name, self.placeholder(Value::String(pattern.clone()), inline))
            }

            ConditionExpression::NotLike(col, pattern) => {
                format!("{} NOT LIKE {}", col.name, self.placeholder(Value::String(pattern.clone()), inline))
            }

            ConditionExpression::Between(col, low, high) => format!(
                "{} BETWEEN {} AND {}",
                col.name,
                self.placeholder(low.clone(), inline),
                self.placeholder(high.clone(), inline),
            ),

            ConditionExpression::NotBetween(col, low, high) => format!(
                "{} NOT BETWEEN {} AND {}",
                col.name,
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

            ConditionExpression::Compare(left, op, right) => format!(
                "{} {} {}",
                self.expression(left, inline),
                match op {
                    ComparisonOp::Eq => "=",
                    ComparisonOp::Ne => "!=",
                    ComparisonOp::Gt => ">",
                    ComparisonOp::Gte => ">=",
                    ComparisonOp::Lt => "<",
                    ComparisonOp::Lte => "<=",
                },
                self.expression(right, inline),
            ),

            ConditionExpression::InSubquery(col, query) => {
                let sql = self.select(query, inline).unwrap_or_default();
                format!("{} IN ({sql})", col.name)
            }

            ConditionExpression::NotInSubquery(col, query) => {
                let sql = self.select(query, inline).unwrap_or_default();
                format!("{} NOT IN ({sql})", col.name)
            }

            ConditionExpression::Exists(query) => {
                let sql = self.select(query, inline).unwrap_or_default();
                format!("EXISTS ({sql})")
            }

            ConditionExpression::NotExists(query) => {
                let sql = self.select(query, inline).unwrap_or_default();
                format!("NOT EXISTS ({sql})")
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use pillar_core::{
        ast::{
            AggregateFunction, AlterTableStatement, ColumnDefinition,
            CreateTableStatement, DeleteStatement, InsertStatement, Interval,
            Projection, SelectStatement, Statement, TtlClause, UpdateStatement,
            AggregateFn, ColumnType,
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

    #[test]
    fn test_select_all() {
        let (sql, params) = transpile(Statement::select(SelectStatement::new("events")));

        assert_eq!(sql, "SELECT * FROM events");
        assert!(params.is_empty());
    }

    #[test]
    fn test_select_with_filter() {
        let (sql, params) = transpile(Statement::select(
            SelectStatement::new("events")
                .where_clause(ConditionExpression::eq("id", 42i64)),
        ));

        assert_eq!(sql, "SELECT * FROM events WHERE id = $1");
        assert_eq!(params, vec![Value::Int64(42)]);
    }

    #[test]
    fn test_select_limit_offset() {
        let (sql, params) = transpile(Statement::select(
            SelectStatement::new("events")
                .limit(10u64)
                .offset(20u64),
        ));

        assert_eq!(sql, "SELECT * FROM events LIMIT 10 OFFSET 20");
        assert!(params.is_empty());
    }

    #[test]
    fn test_insert_single_row() {
        let (sql, params) = transpile(Statement::insert(
            InsertStatement::new("events")
                .columns(["id", "name", "count"])
                .values([[
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
        let (sql, params) = transpile(Statement::insert(
            InsertStatement::new("events")
                .columns(["id", "name"])
                .values([
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
        let (sql, params) = transpile(Statement::update(
            UpdateStatement::new("events")
                .set([("count", Value::Int32(99))])
                .where_clause(ConditionExpression::eq("id", 1i64)),
        ));

        assert_eq!(sql, "UPDATE events SET count = $1 WHERE id = $2");
        assert_eq!(params, vec![Value::Int32(99), Value::Int64(1)]);
    }

    #[test]
    fn test_delete_with_filter() {
        let (sql, params) = transpile(Statement::delete(
            DeleteStatement::new("events")
                .where_clause(ConditionExpression::eq("id", 1i64)),
        ));

        assert_eq!(sql, "DELETE FROM events WHERE id = $1");
        assert_eq!(params, vec![Value::Int64(1)]);
    }

    #[test]
    fn test_delete_all() {
        let (sql, params) = transpile(Statement::delete(DeleteStatement::new("events")));

        assert_eq!(sql, "DELETE FROM events");
        assert!(params.is_empty());
    }

    #[test]
    fn test_condition_and() {
        let (sql, params) = transpile(Statement::select(
            SelectStatement::new("events").where_clause(
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
        let result = t.transpile(&Statement::insert(
            InsertStatement::new("events")
                .columns(["id"])
                .values(Vec::<Vec<Value>>::new()),
        ));

        assert!(result.is_err());
    }

    #[test]
    fn test_column_types_map_correctly() {
        let (sql, params) = transpile(Statement::create_table(
            CreateTableStatement::new("events")
                .columns([
                    col("id", ColumnType::UInt64),
                    col("name", ColumnType::LowCardinalityString),
                    col("event_time", ColumnType::datetime64(3)),
                    col("tag", ColumnType::fixed_string(16)),
                ]),
        ));
        assert_eq!(sql, "CREATE TABLE events (id UBIGINT NOT NULL, name VARCHAR NOT NULL, event_time TIMESTAMP NOT NULL, tag CHAR(16) NOT NULL)");
        assert!(params.is_empty());
    }

    #[test]
    fn test_aggregate_state_column_maps_to_result_type() {
        let (sql, params) = transpile(Statement::create_table(
            CreateTableStatement::new("events_by_minute")
                .columns([
                    col("count", ColumnType::aggregate_state(AggregateFn::count(), [ColumnType::UInt64])),
                    col("duration_avg", ColumnType::aggregate_state(AggregateFn::avg(), [ColumnType::Float64])),
                    col("duration_p95", ColumnType::aggregate_state(AggregateFn::quantile(0.95), [ColumnType::Float64])),
                ]),
        ));
        assert_eq!(sql, "CREATE TABLE events_by_minute (count UBIGINT NOT NULL, duration_avg DOUBLE NOT NULL, duration_p95 DOUBLE NOT NULL)");
        assert!(params.is_empty());
    }

    #[test]
    fn test_ttl_is_ignored() {
        let (sql, params) = transpile(Statement::create_table(
            CreateTableStatement::new("events")
                .columns([col("event_time", ColumnType::datetime64(3))])
                .ttl(TtlClause::delete("event_time", Interval::days(90))),
        ));
        assert_eq!(sql, "CREATE TABLE events (event_time TIMESTAMP NOT NULL)");
        assert!(params.is_empty());
    }

    #[test]
    fn test_alter_table_ttl_only_is_noop() {
        let (sql, params) = transpile(Statement::alter_table(
            AlterTableStatement::new("events")
                .ttl(TtlClause::delete("event_time", Interval::days(30))),
        ));
        assert_eq!(sql, "");
        assert!(params.is_empty());
    }

    #[test]
    fn test_state_merge_emit_plain_function() {
        let (sql, params) = transpile(Statement::select(
            SelectStatement::new("events")
                .projections([
                    Projection::aggregate(AggregateFunction::state(AggregateFunction::count_all())),
                    Projection::aggregate(AggregateFunction::merge(AggregateFunction::avg("duration_ms"))),
                ]),
        ));
        assert_eq!(sql, "SELECT CAST(COUNT(*) AS UBIGINT), ANY_VALUE(duration_ms) FROM events");
        assert!(params.is_empty());
    }

    #[test]
    fn test_merge_projections_with_group_by() {
        let (sql, params) = transpile(Statement::select(
            SelectStatement::new("crawl_log_latency")
                .projections([
                    Projection::column("tenant_id"),
                    Projection::column("bucket"),
                    Projection::aggregate(
                        AggregateFunction::merge(AggregateFunction::quantile(0.5, "latency_p50"))
                    ).alias("latency_p50"),
                    Projection::aggregate(
                        AggregateFunction::merge(AggregateFunction::sum("latency_sum"))
                    ).alias("latency_sum"),
                ])
                .group_by(["tenant_id", "bucket"]),
        ));

        assert_eq!(
            sql,
            "SELECT tenant_id, bucket, ANY_VALUE(latency_p50) AS latency_p50, ANY_VALUE(latency_sum) AS latency_sum FROM crawl_log_latency GROUP BY tenant_id, bucket",
        );
        assert!(params.is_empty());
    }

    #[test]
    fn test_mixed_merge_and_aggregate_with_group_by() {
        let (sql, params) = transpile(Statement::select(
            SelectStatement::new("crawl_log_latency")
                .projections([
                    Projection::column("tenant_id"),
                    Projection::column("step"),
                    Projection::aggregate(
                        AggregateFunction::merge(AggregateFunction::quantile(0.5, "latency_p50"))
                    ).alias("latency_p50"),
                    Projection::aggregate(AggregateFunction::sum("latency_sum")).alias("latency_sum"),
                    Projection::aggregate(AggregateFunction::count_all()).alias("event_count"),
                ])
                .group_by(["tenant_id", "step"]),
        ));

        assert_eq!(
            sql,
            "SELECT tenant_id, step, ANY_VALUE(latency_p50) AS latency_p50, CAST(SUM(latency_sum) AS UBIGINT) AS latency_sum, CAST(COUNT(*) AS UBIGINT) AS event_count FROM crawl_log_latency GROUP BY tenant_id, step",
        );
        assert!(params.is_empty());
    }

    #[test]
    fn test_merge_projections_no_group_by() {
        let (sql, params) = transpile(Statement::select(
            SelectStatement::new("crawl_log_latency")
                .projections([
                    Projection::aggregate(
                        AggregateFunction::merge(AggregateFunction::sum("latency_sum"))
                    ).alias("latency_sum"),
                ]),
        ));

        assert_eq!(
            sql,
            "SELECT ANY_VALUE(latency_sum) AS latency_sum FROM crawl_log_latency",
        );
        assert!(params.is_empty());
    }

    #[test]
    fn test_quantile_maps_to_percentile_cont() {
        let (sql, params) = transpile(Statement::select(
            SelectStatement::new("events")
                .projections([Projection::aggregate(AggregateFunction::quantile(0.95, "duration_ms"))]),
        ));
        assert_eq!(sql, "SELECT PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY duration_ms) FROM events");
        assert!(params.is_empty());
    }
}
