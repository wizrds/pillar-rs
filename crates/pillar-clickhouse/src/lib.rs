//! ClickHouse backend for the pillar query framework.

#[allow(unused_extern_crates)]
extern crate self as pillar_clickhouse;

mod database;
mod dialect;
mod normalize;
mod transpile;

pub use database::{ClickHouseDatabase, ClickHouseDatabaseBuilder};
pub use dialect::ClickHouseDialect;
