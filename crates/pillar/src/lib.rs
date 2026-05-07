//! Pillar is a database-agnostic query and schema framework for Rust.
//!
//! This crate re-exports everything from [`pillar_core`] and provides optional backend
//! integrations and the [`prelude`] module for convenient glob imports.

#[allow(unused_extern_crates)]
extern crate self as pillar;

pub mod prelude;

pub mod macros {
    pub use pillar_macros::*;
}

pub use pillar_core::*;

#[cfg(feature = "duckdb")]
pub mod duckdb {
    pub use pillar_duckdb::*;
}

#[cfg(feature = "clickhouse")]
pub mod clickhouse {
    pub use pillar_clickhouse::*;
}
