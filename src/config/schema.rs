use schemars::schema_for;

use crate::config::types::GlobalConfig;

pub fn generate_schema() -> String {
    let schema = schema_for!(GlobalConfig);
    serde_json::to_string_pretty(&schema).expect("Failed to serialize schema")
}
