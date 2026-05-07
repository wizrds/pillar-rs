# Pillar

A database-agnostic query layer for Rust, built on Apache Arrow. Define your schema once and run it against DuckDB or ClickHouse without changing your application code.

## Quick Start

Add the git repo to your `Cargo.toml`:

```toml
[dependencies]
pillar = { git = "https://github.com/wizrds/pillar-rs", tag = "0.1.0", features = ["uuid", "chrono", "duckdb-bundled"] }
```

Available features:
- `duckdb` / `duckdb-bundled` - DuckDB backend (bundled build includes the DuckDB C library, no system dependency)
- `clickhouse` - ClickHouse backend
- `chrono` - `Date`, `Time`, `DateTime` column types
- `uuid` - `Uuid` column type

## Usage

### Defining a model

Place your `#[derive(Model)]` struct inside a module. The macro generates a `Model` struct and a `Column` struct with typed accessors, all in the same module scope, which are then used to perform queries and operations on the database.

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

The `#[pillar(table = "...")]` attribute sets the table name. If omitted it defaults to the snake_case of the struct name. Fields can be annotated with:
- `#[pillar(primary_key)]` marks the field as a primary key
- `#[pillar(unique)]` marks the field as unique
- `#[pillar(column = "custom_name")]` overrides the column name
- `#[pillar(skip)]` excludes the field from the schema

### Connecting to a database

**DuckDB:**

```rust
use pillar::duckdb::DuckDbDatabase;

let database = DuckDbDatabase::in_memory().await?;
// or: DuckDbDatabase::open("path/to/file.database").await?
```

**ClickHouse:**

```rust
use pillar::clickhouse::ClickHouseDatabase;

let database = ClickHouseDatabase::builder()
    .url("http://localhost:8123")
    .database("my_db")
    .build()
    .await?;
```

### Schema management

Use the AST types directly to create tables:

```rust
use pillar::prelude::*;

database
    .execute(
        &Statement::CreateTable(
            CreateTableStatement::new("events")
                .if_not_exists()
                .columns(vec![
                    ColumnDefinition::new("id", ColumnType::Uuid).primary_key(),
                    ColumnDefinition::new("name", ColumnType::String),
                    ColumnDefinition::new("severity", ColumnType::Int32),
                    ColumnDefinition::new("occurred_at", ColumnType::DateTime),
                ]),
        )
    )
    .await?;
```

TTL is a first-class citizen and is emitted natively on ClickHouse and silently ignored on DuckDB:

```rust
CreateTableStatement::new("events")
    .columns(vec![/* ... */])
    .ttl(TtlClause::delete(
        "occurred_at",
        Interval::new(90, IntervalUnit::Day),
    ));
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
        op
            .create_table(
                CreateTableStatement::new("events")
                    .columns(vec![/* ... */]),
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

### Inserting

```rust
events::Model::insert_batch(vec![
    events::Model { id: Uuid::new_v4(), name: "login".into(), severity: 1, occurred_at: Utc::now() },
    events::Model { id: Uuid::new_v4(), name: "failed_login".into(), severity: 3, occurred_at: Utc::now() },
])
.expect("failed to build insert")
.execute(&database)
.await?;
```

### Querying

```rust
// All rows
let all = events::Model::find()
    .all(&database)
    .await?;

// With a typed filter
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
let mut stream = events::Model::find()
    .stream(&database)
    .await?;

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
// Delete with a filter
events::Model::delete()
    .filter_expr(events::Column::severity().lt(3i32))
    .execute(&database)
    .await?;

// Delete all rows
events::Model::delete_all()
    .execute(&database)
    .await?;
```

### Materialized views

Define a materialized view the same way as a model using `#[derive(MaterializedView)]`. The `from` and `filter` attributes bake a base query into the view definition.

```rust
pub mod high_severity_events {
    use pillar::prelude::*;

    #[derive(MaterializedView)]
    #[pillar(view = "high_severity_events", from = "events", filter = "severity >= 3")]
    pub struct HighSeverityEvent {
        pub id: uuid::Uuid,
        pub name: String,
        pub severity: i32,
        pub occurred_at: chrono::DateTime<chrono::Utc>,
    }
}
```

Instead of using the `from` and `filter` attributes, you can also define the query manually by implementing `ViewQuery` for the view struct:

```rust
use pillar::prelude::*;

impl ViewQuery for high_severity_events::View {
    fn query() -> SelectStatement {
        SelectStatement::new(TableRef::new("events"))
            .projections(vec![
                Projection::Column("id".into()),
                Projection::Column("name".into()),
                Projection::Column("severity".into()),
                Projection::Column("occurred_at".into()),
            ])
            .where_clause(ConditionExpression::gte("severity", 3i32))
    }
}
```

Create the view in the database:

```rust
database
    .execute(&high_severity_events::View::create_statement())
    .await?;
```

Query it with additional callsite filters:

```rust
let results = high_severity_events::View::find()
    .filter_expr(high_severity_events::Column::name().eq("brute_force_attempt"))
    .order_by_desc("severity")
    .all(&database)
    .await?;
```

### Typed column accessors

`Column` provides a typed accessor for every field. Each accessor returns a `TypedColumn<T>` which carries the column name and exposes comparison methods that return a `ConditionExpression`:

```rust
events::Column::severity().eq(3i32)
events::Column::severity().gte(3i32)
events::Column::severity().lt(5i32)
events::Column::name().like("%login%")
events::Column::id().is_null()
```

## License

This project is licensed under the ISC License.

## Support & Feedback

If you encounter any issues or have feedback, please open an issue.

Made with ❤️ by Tim Pogue
