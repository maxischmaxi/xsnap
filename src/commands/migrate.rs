use std::path::{Path, PathBuf};

/// Options for the `xsnap migrate` command.
pub struct MigrateOptions {
    pub source: String,
    pub target: String,
}

/// Run the migrate command.
///
/// Converts OSnap YAML configuration and test files to xsnap JSON format.
///
/// 1. Looks for `osnap.config.yaml` in the source directory and converts it
///    to `xsnap.config.jsonc` in the target directory.
/// 2. Looks for `*.osnap.yaml` test files and converts them to `*.xsnap.json`.
/// 3. Uses `dialoguer::Confirm` for each file to get user confirmation.
pub fn run_migrate(opts: MigrateOptions) -> anyhow::Result<()> {
    let source_dir = Path::new(&opts.source);
    let target_dir = Path::new(&opts.target);

    if !source_dir.exists() {
        anyhow::bail!("Source directory does not exist: {}", source_dir.display());
    }

    std::fs::create_dir_all(target_dir)?;

    let mut migrated_count = 0;
    let mut skipped_count = 0;

    // 1. Look for osnap.config.yaml -> convert to xsnap.config.jsonc
    let config_source = source_dir.join("osnap.config.yaml");
    if config_source.exists() {
        let config_target = target_dir.join("xsnap.config.jsonc");

        let prompt = format!(
            "Migrate {} -> {}?",
            config_source.display(),
            config_target.display()
        );
        let should_migrate = dialoguer::Confirm::new()
            .with_prompt(&prompt)
            .default(true)
            .interact()
            .unwrap_or(false);

        if should_migrate {
            migrate_yaml_file(&config_source, &config_target)?;
            println!("  Migrated: {}", config_target.display());
            migrated_count += 1;
        } else {
            println!("  Skipped: {}", config_source.display());
            skipped_count += 1;
        }
    } else {
        println!("No osnap.config.yaml found in {}", source_dir.display());
    }

    // 2. Look for *.osnap.yaml test files -> convert to *.xsnap.json
    let test_files = find_osnap_test_files(source_dir)?;

    if test_files.is_empty() {
        println!("No *.osnap.yaml test files found.");
    } else {
        println!("\nFound {} OSnap test file(s):", test_files.len());

        for test_file in &test_files {
            let relative = test_file.strip_prefix(source_dir).unwrap_or(test_file);

            // Convert filename: foo.osnap.yaml -> foo.xsnap.json
            let new_name = relative
                .to_string_lossy()
                .replace(".osnap.yaml", ".xsnap.json")
                .replace(".osnap.yml", ".xsnap.json");
            let target_path = target_dir.join(&new_name);

            let prompt = format!(
                "Migrate {} -> {}?",
                test_file.display(),
                target_path.display()
            );
            let should_migrate = dialoguer::Confirm::new()
                .with_prompt(&prompt)
                .default(true)
                .interact()
                .unwrap_or(false);

            if should_migrate {
                // Ensure target directory exists.
                if let Some(parent) = target_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }

                migrate_yaml_file(test_file, &target_path)?;
                println!("  Migrated: {}", target_path.display());
                migrated_count += 1;
            } else {
                println!("  Skipped: {}", test_file.display());
                skipped_count += 1;
            }
        }
    }

    println!(
        "\nMigration complete: {} migrated, {} skipped.",
        migrated_count, skipped_count
    );

    Ok(())
}

/// Find all *.osnap.yaml and *.osnap.yml files recursively in the given directory.
fn find_osnap_test_files(dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let mut results = Vec::new();

    let yaml_pattern = dir.join("**/*.osnap.yaml").display().to_string();
    let yml_pattern = dir.join("**/*.osnap.yml").display().to_string();

    for pattern in &[yaml_pattern, yml_pattern] {
        for path in glob::glob(pattern)?.flatten() {
            results.push(path);
        }
    }

    results.sort();
    Ok(results)
}

/// Reads a YAML file, converts it to JSON, and writes the result.
///
/// The conversion is straightforward: parse YAML into a `serde_yaml::Value`,
/// convert to `serde_json::Value`, then pretty-print as JSON.
fn migrate_yaml_file(source: &Path, target: &Path) -> anyhow::Result<()> {
    let content = std::fs::read_to_string(source)?;
    let yaml_value: serde_yaml::Value = serde_yaml::from_str(&content)?;
    let json_value = yaml_to_json(&yaml_value);
    let json_string = serde_json::to_string_pretty(&json_value)?;
    std::fs::write(target, json_string)?;
    Ok(())
}

/// Recursively converts a `serde_yaml::Value` to a `serde_json::Value`.
fn yaml_to_json(yaml: &serde_yaml::Value) -> serde_json::Value {
    match yaml {
        serde_yaml::Value::Null => serde_json::Value::Null,
        serde_yaml::Value::Bool(b) => serde_json::Value::Bool(*b),
        serde_yaml::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                serde_json::Value::Number(i.into())
            } else if let Some(u) = n.as_u64() {
                serde_json::Value::Number(u.into())
            } else if let Some(f) = n.as_f64() {
                serde_json::Number::from_f64(f)
                    .map(serde_json::Value::Number)
                    .unwrap_or(serde_json::Value::Null)
            } else {
                serde_json::Value::Null
            }
        }
        serde_yaml::Value::String(s) => serde_json::Value::String(s.clone()),
        serde_yaml::Value::Sequence(seq) => {
            let arr: Vec<serde_json::Value> = seq.iter().map(yaml_to_json).collect();
            serde_json::Value::Array(arr)
        }
        serde_yaml::Value::Mapping(map) => {
            let mut obj = serde_json::Map::new();
            for (k, v) in map {
                let key = match k {
                    serde_yaml::Value::String(s) => s.clone(),
                    other => format!("{:?}", other),
                };
                obj.insert(key, yaml_to_json(v));
            }
            serde_json::Value::Object(obj)
        }
        serde_yaml::Value::Tagged(tagged) => {
            // Convert the inner value, ignoring the YAML tag.
            yaml_to_json(&tagged.value)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yaml_to_json_null() {
        let yaml = serde_yaml::Value::Null;
        let json = yaml_to_json(&yaml);
        assert_eq!(json, serde_json::Value::Null);
    }

    #[test]
    fn test_yaml_to_json_bool() {
        let yaml = serde_yaml::Value::Bool(true);
        let json = yaml_to_json(&yaml);
        assert_eq!(json, serde_json::Value::Bool(true));
    }

    #[test]
    fn test_yaml_to_json_string() {
        let yaml = serde_yaml::Value::String("hello".into());
        let json = yaml_to_json(&yaml);
        assert_eq!(json, serde_json::Value::String("hello".into()));
    }

    #[test]
    fn test_yaml_to_json_number_int() {
        let yaml: serde_yaml::Value = serde_yaml::from_str("42").unwrap();
        let json = yaml_to_json(&yaml);
        assert_eq!(json, serde_json::json!(42));
    }

    #[test]
    fn test_yaml_to_json_number_float() {
        let yaml: serde_yaml::Value = serde_yaml::from_str("3.14").unwrap();
        let json = yaml_to_json(&yaml);
        assert_eq!(json, serde_json::json!(3.14));
    }

    #[test]
    fn test_yaml_to_json_sequence() {
        let yaml: serde_yaml::Value = serde_yaml::from_str("[1, 2, 3]").unwrap();
        let json = yaml_to_json(&yaml);
        assert_eq!(json, serde_json::json!([1, 2, 3]));
    }

    #[test]
    fn test_yaml_to_json_mapping() {
        let yaml: serde_yaml::Value = serde_yaml::from_str("key: value\nnum: 42").unwrap();
        let json = yaml_to_json(&yaml);
        assert_eq!(json, serde_json::json!({"key": "value", "num": 42}));
    }

    #[test]
    fn test_yaml_to_json_nested() {
        let yaml_str = r#"
name: test
sizes:
  - name: desktop
    width: 1920
    height: 1080
"#;
        let yaml: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
        let json = yaml_to_json(&yaml);
        assert_eq!(json["name"], serde_json::json!("test"));
        assert_eq!(json["sizes"][0]["name"], serde_json::json!("desktop"));
        assert_eq!(json["sizes"][0]["width"], serde_json::json!(1920));
    }
}
