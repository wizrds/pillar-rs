use pillar_core::{
    ast::Statement,
    dialect::{Dialect, Feature, PreparedStatement},
    errors::Error,
};

use crate::transpile::Transpiler;


/// A [`pillar_core::dialect::Dialect`](pillar_core::dialect::Dialect) implementation for ClickHouse.
pub struct ClickHouseDialect;

impl Dialect for ClickHouseDialect {
    fn name(&self) -> &'static str {
        "clickhouse"
    }

    fn transpile(&self, statement: &Statement) -> Result<PreparedStatement, Error> {
        let mut transpiler = Transpiler::new();
        transpiler
            .transpile(statement)
            .map(|sql| transpiler.finish(sql))
    }

    fn supports_feature(&self, feature: Feature) -> bool {
        matches!(
            feature,
            Feature::WindowFunctions
                | Feature::CommonTableExpressions
                | Feature::MaterializedViews
                | Feature::Partitioning
                | Feature::ArrayFunctions
                | Feature::JsonFunctions
                | Feature::ApproximateAggregates
                | Feature::NestedTypes,
        )
    }

    fn quote_identifier(&self, id: &str) -> String {
        format!("`{}`", id.replace('`', "``"))
    }

    fn parameter_placeholder(&self, _index: usize) -> String {
        "?".to_string()
    }
}
