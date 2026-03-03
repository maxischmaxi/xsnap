use schemars::schema_for;

use crate::config::types::{GlobalConfig, TestFile};

pub fn generate_schema() -> String {
    let schema = schema_for!(GlobalConfig);
    serde_json::to_string_pretty(&schema).expect("Failed to serialize schema")
}

pub fn generate_test_schema() -> String {
    let schema = schema_for!(TestFile);
    serde_json::to_string_pretty(&schema).expect("Failed to serialize test schema")
}
