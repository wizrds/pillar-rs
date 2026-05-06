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
