//! DuckDB backend for the pillar query framework.

#[allow(unused_extern_crates)]
extern crate self as pillar_duckdb;

mod database;
mod dialect;
mod normalize;
mod transpile;
mod value;

pub use database::DuckDbDatabase;
pub use dialect::DuckDbDialect;
