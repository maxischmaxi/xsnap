use std::collections::HashSet;

use crate::config::types::{Action, GlobalConfig, TestConfig};
use crate::error::XsnapError;

pub fn validate_config(
    global: &GlobalConfig,
    tests: &[TestConfig],
) -> Result<(), XsnapError> {
    validate_unique_names(tests)?;
    validate_function_references(global, tests)?;
    Ok(())
}

fn validate_unique_names(tests: &[TestConfig]) -> Result<(), XsnapError> {
    let mut seen = HashSet::new();
    for test in tests {
        if !seen.insert(&test.name) {
            return Err(XsnapError::DuplicateTestName {
                name: test.name.clone(),
            });
        }
    }
    Ok(())
}

fn validate_function_references(
    global: &GlobalConfig,
    tests: &[TestConfig],
) -> Result<(), XsnapError> {
    for test in tests {
        if let Some(actions) = &test.actions {
            for action in actions {
                if let Action::Function { name, .. } = action
                    && !global.functions.contains_key(name) {
                        return Err(XsnapError::UndefinedFunction {
                            name: name.clone(),
                        });
                    }
            }
        }
    }
    Ok(())
}
