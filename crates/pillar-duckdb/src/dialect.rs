use pillar_core::{
    ast::Statement,
    errors::Error,
    dialect::{Dialect, Feature, PreparedStatement},
};

use crate::transpile::Transpiler;


pub struct DuckDbDialect;

impl Dialect for DuckDbDialect {
    fn name(&self) -> &'static str {
        "duckdb"
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
        format!("\"{}\"", id.replace('"', "\"\""))
    }

    fn parameter_placeholder(&self, index: usize) -> String {
        format!("${index}")
    }
}
