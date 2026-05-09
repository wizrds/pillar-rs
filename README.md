# Pillar

A database-agnostic query layer for Rust, built on Apache Arrow. Define your schema once and run it against DuckDB or ClickHouse without changing your application code.

## Quick Start

Add the git repo to your `Cargo.toml`:

```toml
[dependencies]
pillar = { git = "https://github.com/wizrds/pillar-rs", tag = "0.2.1", features = ["uuid", "chrono", "duckdb-bundled"] }
```

Available features:
- `duckdb` / `duckdb-bundled` - DuckDB backend (bundled build includes the DuckDB C library, no system dependency)
- `clickhouse` - ClickHouse backend
- `chrono` - `Date`, `Time`, `DateTime` column types
- `uuid` - `Uuid` column type

## Usage

### Defining a model

Place your `#[derive(Model)]` struct inside a module. The macro generates a `Model` struct and a `Column` struct with typed accessors in the same module scope.

```rust
pub mod events {
    use pillar::prelude::*;

    #[derive(Model)]
    #[pillar(table = "events")]
    pub struct Event {
        #[pillar(primary_key)]
        pub id: uuid::Uuid,
        pub name: String,
        pub severity: i32,
        pub occurred_at: chrono::DateTime<chrono::Utc>,
    }
}
```

#### Model struct attributes

| Attribute | Type | Description |
|---|---|---|
| `table` | `String` | Table name. Defaults to the snake_case struct name. |
| `engine` | `String` | Storage engine (e.g. `"MergeTree"`). Passed through to the backend; ignored by backends that do not use it. |
| `partition_by` | `String` | Partition key expression (e.g. `"toYYYYMM(occurred_at)"`). |
| `ttl` | sub-attrs | TTL rule for automatic row expiry. See below. |
| `options` | key-value map | Catch-all for any backend-specific option not covered by a named field. |

#### Model field attributes

| Attribute | Type | Description |
|---|---|---|
| `primary_key` | flag | Marks the field as a primary key. |
| `unique` | flag | Marks the field as unique. |
| `column` | `String` | Overrides the column name. |
| `column_type` | `String` | Overrides the inferred column type with a raw backend type string. |
| `order_by` | flag | Includes this field in the ORDER BY key of the generated `create_statement`. |
| `skip` | flag | Excludes the field from the schema entirely. |

#### TTL

TTL is expressed as a strongly typed sub-attribute on the struct:

```rust
#[derive(Model)]
#[pillar(
    table = "events",
    engine = "MergeTree",
    partition_by = "toYYYYMM(occurred_at)",
    ttl(column = "occurred_at", interval = 90, unit = "day")
)]
pub struct Event {
    #[pillar(order_by)]
    pub id: uuid::Uuid,
    pub name: String,
    pub severity: i32,
    #[pillar(order_by)]
    pub occurred_at: chrono::DateTime<chrono::Utc>,
}
```

Valid `unit` values: `"second"`, `"minute"`, `"hour"`, `"day"`, `"week"`, `"month"`, `"year"` (singular or plural).

#### Schema provisioning

When DDL attributes (`engine`, `partition_by`, `ttl`, `order_by`, or `options`) are present, the macro generates an impl of `TableSchema` for the model, which provides `create_statement()`. Without DDL attributes a blanket impl produces a plain `CREATE TABLE` from the column definitions.

```rust
database.execute(&events::Model::create_statement()).await?;
```

#### Backend-specific options

Use `options` as a catch-all for any backend-specific key-value pair not covered by a named attribute:

```rust
#[pillar(
    table = "events",
    engine = "MergeTree",
    options(index_granularity = "8192")
)]
pub struct Event { /* ... */ }
```

### Connecting to a database

**DuckDB:**

```rust
use pillar::duckdb::DuckDbDatabase;

let database = DuckDbDatabase::in_memory().await?;
// or: DuckDbDatabase::open("path/to/file.db").await?
```

**ClickHouse:**

```rust
use pillar::clickhouse::ClickHouseDatabase;

let database = ClickHouseDatabase::builder("http://localhost:8123")
    .database("my_db")
    .username("default")
    .password("secret")
    .setting("max_execution_time", "30")
    .build();
```

### Migrations

Implement `Migration` for each revision and register them with `Migrations`:

```rust
use pillar::prelude::*;

struct CreateEventsTable;

#[async_trait::async_trait]
impl Migration for CreateEventsTable {
    fn id(&self) -> &'static str { "001_create_events" }
    fn previous_id(&self) -> Option<&'static str> { None }

    async fn up(&self, op: &MigrateOp<'_>) -> Result<(), Error> {
        op.create_table(
            CreateTableStatement::new("events")
                .columns([
                    ColumnDefinition::new("id", ColumnType::Uuid).primary_key(),
                    ColumnDefinition::new("name", ColumnType::String),
                    ColumnDefinition::new("severity", ColumnType::Int32),
                    ColumnDefinition::new("occurred_at", ColumnType::DateTime),
                ]),
        )
        .await?;

        Ok(())
    }

    async fn down(&self, op: &MigrateOp<'_>) -> Result<(), Error> {
        op.drop_table(DropTableStatement::new("events")).await?;
        Ok(())
    }
}

struct MyMigrations;

impl Migrations for MyMigrations {
    fn migrations() -> Vec<MigrationRef> {
        vec![Box::new(CreateEventsTable)]
    }
}

database.upgrade::<MyMigrations>().await?;
database.downgrade_to::<MyMigrations>("001_create_events").await?;
```

Alternatively, models with DDL attributes can produce their own `create_statement()` via `TableSchema`, which can be used directly inside a migration:

```rust
async fn up(&self, op: &MigrateOp<'_>) -> Result<(), Error> {
    op.execute(&events::Model::create_statement()).await?;

    Ok(())
}
```

### Inserting

```rust
events::Model::insert_batch(vec![
    events::Model {
        id: Uuid::new_v4(),
        name: "login".into(),
        severity: 1,
        occurred_at: Utc::now(),
    },
])
.expect("failed to build insert")
.execute(&database)
.await?;
```

### Querying

```rust
// All rows
let all = events::Model::find().all(&database).await?;

// Filtered and ordered
let high = events::Model::find()
    .filter_expr(events::Column::severity().gte(3i32))
    .order_by_desc("severity")
    .limit(10)
    .all(&database)
    .await?;

// Single row
let one = events::Model::find()
    .filter_expr(events::Column::name().eq("login"))
    .one(&database)
    .await?;

// Streaming large result sets
let mut stream = events::Model::find().stream(&database).await?;

while let Some(rows) = stream.try_next().await? {
    // rows is Vec<events::Model>
}
```

### Updating

```rust
events::Model::update()
    .set("severity", 4i32)
    .filter_expr(events::Column::name().eq("failed_login"))
    .execute(&database)
    .await?;
```

### Deleting

```rust
// Filtered delete
events::Model::delete()
    .filter_expr(events::Column::severity().lt(3i32))
    .execute(&database)
    .await?;

// Delete all rows
events::Model::delete_all().execute(&database).await?;
```

### Advanced queries

When the model API is not expressive enough you can build a `Statement` directly using the AST and execute it with `database.query(...)` or `database.execute(...)`. The AST is fully fluent:

```rust
use pillar::prelude::*;

// CTEs
let stmt = Statement::select(
    SelectStatement::new("ranked")
        .with(Cte::new("ranked",
            SelectStatement::new("events")
                .projections([
                    Projection::column("id"),
                    Projection::column("severity"),
                    Projection::expr(Expression::window(
                        WindowFunction::rank(
                            WindowSpec::new()
                                .partition_by(["host"])
                                .order_by([OrderBy::desc("severity")]),
                        ),
                    )).alias("rnk"),
                ])
        ))
        .where_clause(ConditionExpression::lte("rnk", 3i32))
);

// Subqueries
let stmt = Statement::select(
    SelectStatement::new("events")
        .where_clause(ConditionExpression::in_subquery(
            "job_id",
            SelectStatement::new("active_jobs")
                .projections([Projection::column("id")]),
        ))
);

// UNION / INTERSECT / EXCEPT
let stmt = Statement::compound(CompoundSelect::union_all(
    SelectStatement::new("events_2023"),
    SelectStatement::new("events_2024"),
));

// INSERT ... SELECT
let stmt = Statement::insert(
    InsertStatement::new("archive")
        .columns(["id", "name"])
        .select(
            SelectStatement::new("events")
                .where_clause(ConditionExpression::lt("severity", 2i32)),
        )
);

// RETURNING
let stmt = Statement::insert(
    InsertStatement::new("events")
        .columns(["name", "severity"])
        .values([[Value::string("login"), Value::Int32(1)]])
        .returning([Projection::column("id")])
);
```

Everything that compiles against the AST works on both DuckDB and ClickHouse. Features with no equivalent on a given backend are either mapped to the closest available construct or silently omitted (for example, DuckDB maps ClickHouse TTL to a no-op, and ClickHouse does not emit `RETURNING` clauses).

### Views

Define a view with `#[derive(View)]`. The macro generates a `View` struct and a `Column` struct, parallel to how `Model` works. Add `materialized` to emit `CREATE MATERIALIZED VIEW` instead of `CREATE VIEW`.

#### Simple pass-through view

Use `from` and `filter` to bake a query into the view definition:

```rust
pub mod high_severity_events {
    use pillar::prelude::*;

    #[derive(View)]
    #[pillar(view = "high_severity_events", from = "events", filter = "severity >= 3")]
    pub struct HighSeverityEvent {
        pub id: uuid::Uuid,
        pub name: String,
        pub severity: i32,
        pub occurred_at: chrono::DateTime<chrono::Utc>,
    }
}
```

#### Aggregate rollup view

Mark aggregate fields with `#[pillar(aggregate = "...")]`. Non-aggregate fields are automatically used as GROUP BY keys. Use `source` to specify the source column when it differs from the destination field name.

```rust
pub mod events_by_minute {
    use pillar::prelude::*;

    #[derive(View)]
    #[pillar(
        view = "events_by_minute_mv",
        materialized,
        from = "events",
        to = "events_by_minute",
        engine = "SummingMergeTree",
        partition_by = "toYYYYMM(event_time)"
    )]
    pub struct EventByMinute {
        #[pillar(order_by)]
        pub event_time: chrono::DateTime<chrono::Utc>,
        #[pillar(order_by)]
        pub host: String,
        #[pillar(aggregate = "count")]
        pub event_count: u64,
        #[pillar(aggregate = "sum", source = "duration_ms")]
        pub duration_ms_sum: u64,
        #[pillar(aggregate = "sum", source = "bytes")]
        pub bytes_sum: u64,
    }
}
```

Valid `aggregate` values: `"count"`, `"sum"`, `"avg"`, `"min"`, `"max"`.

#### View struct attributes

| Attribute | Type | Description |
|---|---|---|
| `view` | `String` | View name. Defaults to the snake_case struct name. |
| `materialized` | flag | Emits `CREATE MATERIALIZED VIEW` instead of `CREATE VIEW`. |
| `from` | `String` | Source table for the auto-generated `ViewQuery` impl. |
| `filter` | `String` | Optional WHERE filter on the generated query. |
| `to` | `String` | Routes MV output to this table (ClickHouse `TO table`). Ignored by DuckDB. |
| `engine` | `String` | Storage engine for the MV (e.g. `"SummingMergeTree"`). |
| `partition_by` | `String` | Partition key expression for the MV. |
| `options` | key-value map | Catch-all for backend-specific options. |

#### View field attributes

| Attribute | Type | Description |
|---|---|---|
| `column` | `String` | Overrides the column name. |
| `column_type` | `String` | Overrides the inferred column type with a raw backend type string. |
| `order_by` | flag | Includes this field in the ORDER BY key of the generated `create_statement`. |
| `aggregate` | `String` | Aggregate function for this field (`"count"`, `"sum"`, `"avg"`, `"min"`, `"max"`). |
| `source` | `String` | Source column name for the aggregate. Defaults to the field name. |
| `skip` | flag | Excludes the field from the schema. |

#### Schema provisioning

`ViewSchema::create_statement()` is available when any DDL attribute (`materialized`, `engine`, `partition_by`, `to`, or `options`) is set, or when `ViewSchema` is implemented manually.

```rust
database.execute(&events_by_minute::View::create_statement()).await?;
```

#### Manual ViewQuery

When the macro's `from`/`filter`/`aggregate` attributes are not expressive enough, implement `ViewQuery` manually:

```rust
impl ViewQuery for events_by_minute::View {
    fn query() -> SelectStatement {
        SelectStatement::new("events")
            .projections([
                Projection::column("event_time"),
                Projection::column("host"),
                Projection::aggregate(AggregateFunction::count_all()),
                Projection::aggregate(AggregateFunction::sum("duration_ms")),
                Projection::aggregate(AggregateFunction::sum("bytes")),
            ])
            .group_by(["event_time", "host"])
    }
}
```

### Querying views

Views use the same query API as models:

```rust
let results = high_severity_events::View::find()
    .filter_expr(high_severity_events::Column::severity().gte(4i32))
    .order_by_desc("occurred_at")
    .limit(100)
    .all(&database)
    .await?;
```

### Typed column accessors

Every `Model` and `View` gets a `Column` struct with a typed accessor per field:

```rust
events::Column::severity().eq(3i32)
events::Column::severity().gte(3i32)
events::Column::severity().between(1i32, 5i32)
events::Column::name().like("%login%")
events::Column::id().is_null()
events::Column::occurred_at().lt(cutoff)
```

Each accessor returns a `TypedColumn<T>`, which exposes: `eq`, `ne`, `gt`, `gte`, `lt`, `lte`, `in_list`, `is_not_in`, `is_null`, `is_not_null`, `between`, `not_between`, `like`, `not_like`.

### Custom column types

Use `column_type` to pass a raw type string through to the backend for types not in the `ColumnType` enum:

```rust
#[pillar(column_type = "AggregateFunction(sum, UInt64)")]
pub bytes_sum: u64,
```

On backends that do not understand the type, it is emitted verbatim.

## License

This project is licensed under the ISC License.

## Support & Feedback

If you encounter any issues or have feedback, please open an issue.

Made with ❤️ by Tim Pogue
