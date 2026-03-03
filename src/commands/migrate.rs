use std::collections::HashMap;
use std::path::{Path, PathBuf};

use regex::Regex;

/// Options for the `xsnap migrate` command.
pub struct MigrateOptions {
    pub source: String,
    pub target: String,
}

/// A resolved size with name, width, and height.
#[derive(Debug, Clone)]
struct SizeInfo {
    name: String,
    width: u32,
    height: u32,
}

/// Maps built from the OSnap config's defaultSizes.
struct SizeMaps {
    /// "small" → SizeInfo { name: "small", width: 640, height: 360 }
    by_name: HashMap<String, SizeInfo>,
    /// (1920, 1080) → "xlarge"
    by_dimensions: HashMap<(u32, u32), String>,
}

/// Parsed OSnap config data relevant for migration.
struct OsnapConfig {
    size_maps: SizeMaps,
    snapshot_directory: Option<String>,
    /// The full config as JSON (for writing the migrated config file).
    json_value: serde_json::Value,
}

/// Run the migrate command.
///
/// 1. Parses `osnap.config.yaml` to build size maps.
/// 2. Converts config to `xsnap.config.jsonc`.
/// 3. Converts `*.osnap.yaml` test files to `*.xsnap.json` with resolved sizes.
/// 4. Renames snapshot files from OSnap naming to xsnap naming.
pub fn run_migrate(opts: MigrateOptions) -> anyhow::Result<()> {
    let source_dir = Path::new(&opts.source);
    let target_dir = Path::new(&opts.target);

    if !source_dir.exists() {
        anyhow::bail!("Source directory does not exist: {}", source_dir.display());
    }

    std::fs::create_dir_all(target_dir)?;

    let mut migrated_count = 0;
    let mut skipped_count = 0;

    // 1. Parse OSnap config and build size maps
    let config_source = source_dir.join("osnap.config.yaml");
    let osnap_config = if config_source.exists() {
        Some(parse_osnap_config(&config_source)?)
    } else {
        println!("No osnap.config.yaml found in {}", source_dir.display());
        None
    };

    // 2. Migrate config file
    if let Some(ref config) = osnap_config {
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
            let json_string = serde_json::to_string_pretty(&config.json_value)?;
            std::fs::write(&config_target, json_string)?;
            println!("  Migrated: {}", config_target.display());
            migrated_count += 1;
        } else {
            println!("  Skipped: {}", config_source.display());
            skipped_count += 1;
        }
    }

    // 3. Migrate test files with size resolution
    let size_map = osnap_config
        .as_ref()
        .map(|c| &c.size_maps.by_name)
        .cloned()
        .unwrap_or_default();

    let test_files = find_osnap_test_files(source_dir)?;

    if test_files.is_empty() {
        println!("No *.osnap.yaml test files found.");
    } else {
        println!("\nFound {} OSnap test file(s):", test_files.len());

        for test_file in &test_files {
            let relative = test_file.strip_prefix(source_dir).unwrap_or(test_file);

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
                if let Some(parent) = target_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }

                migrate_yaml_test_file(test_file, &target_path, &size_map)?;
                println!("  Migrated: {}", target_path.display());
                migrated_count += 1;
            } else {
                println!("  Skipped: {}", test_file.display());
                skipped_count += 1;
            }
        }
    }

    // 4. Rename snapshot files
    if let Some(ref config) = osnap_config
        && let Some(ref snapshot_dir_rel) = config.snapshot_directory
    {
        let snapshot_dir = source_dir.join(snapshot_dir_rel);
        let base_images_dir = snapshot_dir.join("__base_images__");

        if base_images_dir.exists() {
            let (renamed, rename_skipped) =
                migrate_snapshot_files(&base_images_dir, &config.size_maps.by_dimensions)?;
            migrated_count += renamed;
            skipped_count += rename_skipped;
        } else {
            println!(
                "\nNo __base_images__ directory found at {}",
                base_images_dir.display()
            );
        }
    }

    println!(
        "\nMigration complete: {} migrated, {} skipped.",
        migrated_count, skipped_count
    );

    Ok(())
}

/// Parse the OSnap config file and build size lookup maps.
fn parse_osnap_config(path: &Path) -> anyhow::Result<OsnapConfig> {
    let content = std::fs::read_to_string(path)?;
    let yaml_value: serde_yaml::Value = serde_yaml::from_str(&content)?;
    let json_value = yaml_to_json(&yaml_value);

    let mut by_name: HashMap<String, SizeInfo> = HashMap::new();
    let mut by_dimensions: HashMap<(u32, u32), String> = HashMap::new();

    if let Some(sizes) = json_value.get("defaultSizes").and_then(|v| v.as_array()) {
        for size in sizes {
            let name = size.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let width = size.get("width").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let height = size.get("height").and_then(|v| v.as_u64()).unwrap_or(0) as u32;

            if !name.is_empty() && width > 0 && height > 0 {
                let info = SizeInfo {
                    name: name.to_string(),
                    width,
                    height,
                };
                by_name.insert(name.to_string(), info);
                by_dimensions.insert((width, height), name.to_string());
            }
        }
    }

    let snapshot_directory = json_value
        .get("snapshotDirectory")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    Ok(OsnapConfig {
        size_maps: SizeMaps {
            by_name,
            by_dimensions,
        },
        snapshot_directory,
        json_value,
    })
}

/// Resolve string-based sizes in a JSON value using the size map.
///
/// If `sizes` is an array of strings like `["small", "large"]`, each string
/// is replaced with an object `{"name": "small", "width": 640, "height": 360}`.
/// If `sizes` is already an array of objects, it is left unchanged.
fn resolve_sizes(value: &mut serde_json::Value, size_map: &HashMap<String, SizeInfo>) {
    if let Some(obj) = value.as_object_mut()
        && let Some(sizes_val) = obj.get_mut("sizes")
        && let Some(sizes_arr) = sizes_val.as_array()
    {
        let needs_resolution = sizes_arr.iter().any(|item| item.is_string());

        if needs_resolution {
            let mut resolved = Vec::new();
            for item in sizes_arr {
                if let Some(name) = item.as_str() {
                    if let Some(info) = size_map.get(name) {
                        resolved.push(serde_json::json!({
                            "name": info.name,
                            "width": info.width,
                            "height": info.height,
                        }));
                    } else {
                        println!("  Warning: Unknown size '{}' — skipping resolution", name);
                        resolved.push(item.clone());
                    }
                } else {
                    // Already an object, keep as-is
                    resolved.push(item.clone());
                }
            }
            *sizes_val = serde_json::Value::Array(resolved);
        }
    }
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

/// Reads a YAML test file, converts it to xsnap JSON format with resolved sizes.
fn migrate_yaml_test_file(
    source: &Path,
    target: &Path,
    size_map: &HashMap<String, SizeInfo>,
) -> anyhow::Result<()> {
    let content = std::fs::read_to_string(source)?;
    let yaml_value: serde_yaml::Value = serde_yaml::from_str(&content)?;
    let mut json_value = yaml_to_json(&yaml_value);

    // Resolve sizes in each test entry
    if let Some(arr) = json_value.as_array_mut() {
        for test in arr.iter_mut() {
            resolve_sizes(test, size_map);
        }
    } else if json_value.is_object() {
        resolve_sizes(&mut json_value, size_map);
    }

    let wrapped = if json_value.is_array() {
        serde_json::json!({
            "$schema": "https://raw.githubusercontent.com/maxischmaxi/xsnap/main/xsnap.test.schema.json",
            "tests": json_value
        })
    } else {
        json_value
    };

    let json_string = serde_json::to_string_pretty(&wrapped)?;
    std::fs::write(target, json_string)?;
    Ok(())
}

/// Parse an OSnap snapshot filename like `Accordion--default_1920x1080.png`
/// into (test_name, width, height).
fn parse_osnap_filename(filename: &str) -> Option<(String, u32, u32)> {
    let re = Regex::new(r"^(.+)_(\d+)x(\d+)\.png$").ok()?;
    let caps = re.captures(filename)?;

    let name = caps.get(1)?.as_str().to_string();
    let width: u32 = caps.get(2)?.as_str().parse().ok()?;
    let height: u32 = caps.get(3)?.as_str().parse().ok()?;

    Some((name, width, height))
}

/// Rename snapshot files from OSnap naming to xsnap naming.
///
/// OSnap: `{name}_{width}x{height}.png`
/// xsnap: `{name}-{size_name}-{width}x{height}.png`
fn migrate_snapshot_files(
    base_images_dir: &Path,
    reverse_size_map: &HashMap<(u32, u32), String>,
) -> anyhow::Result<(usize, usize)> {
    let pattern = base_images_dir.join("*.png").display().to_string();
    let files: Vec<PathBuf> = glob::glob(&pattern)?.flatten().collect();

    if files.is_empty() {
        println!(
            "\nNo .png snapshot files found in {}",
            base_images_dir.display()
        );
        return Ok((0, 0));
    }

    println!(
        "\nFound {} snapshot file(s) in {}:",
        files.len(),
        base_images_dir.display()
    );

    let mut renamed_count = 0;
    let mut skipped_count = 0;

    for file in &files {
        let filename = match file.file_name().and_then(|f| f.to_str()) {
            Some(f) => f,
            None => continue,
        };

        let (test_name, width, height) = match parse_osnap_filename(filename) {
            Some(parsed) => parsed,
            None => {
                println!("  Warning: Could not parse filename '{}'", filename);
                skipped_count += 1;
                continue;
            }
        };

        let size_name = match reverse_size_map.get(&(width, height)) {
            Some(name) => name,
            None => {
                println!(
                    "  Warning: No size name for {}x{} in '{}' — skipping",
                    width, height, filename
                );
                skipped_count += 1;
                continue;
            }
        };

        let new_filename = format!("{}-{}-{}x{}.png", test_name, size_name, width, height);
        let new_path = base_images_dir.join(&new_filename);

        if new_path.exists() {
            println!("  Skipped (already exists): {}", new_filename);
            skipped_count += 1;
            continue;
        }

        let prompt = format!("Rename {} -> {}?", filename, new_filename);
        let should_rename = dialoguer::Confirm::new()
            .with_prompt(&prompt)
            .default(true)
            .interact()
            .unwrap_or(false);

        if should_rename {
            std::fs::rename(file, &new_path)?;
            println!("  Renamed: {}", new_filename);
            renamed_count += 1;
        } else {
            println!("  Skipped: {}", filename);
            skipped_count += 1;
        }
    }

    Ok((renamed_count, skipped_count))
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
        serde_yaml::Value::Tagged(tagged) => yaml_to_json(&tagged.value),
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

    #[test]
    fn test_parse_osnap_filename() {
        let result = parse_osnap_filename("Accordion--default_1920x1080.png");
        assert!(result.is_some());
        let (name, width, height) = result.unwrap();
        assert_eq!(name, "Accordion--default");
        assert_eq!(width, 1920);
        assert_eq!(height, 1080);
    }

    #[test]
    fn test_parse_osnap_filename_complex_name() {
        let result = parse_osnap_filename("My-Component--variant_375x211.png");
        assert!(result.is_some());
        let (name, width, height) = result.unwrap();
        assert_eq!(name, "My-Component--variant");
        assert_eq!(width, 375);
        assert_eq!(height, 211);
    }

    #[test]
    fn test_parse_osnap_filename_invalid() {
        assert!(parse_osnap_filename("not-a-snapshot.png").is_none());
        assert!(parse_osnap_filename("file.txt").is_none());
    }

    #[test]
    fn test_resolve_sizes_string_array() {
        let mut size_map = HashMap::new();
        size_map.insert(
            "small".to_string(),
            SizeInfo {
                name: "small".to_string(),
                width: 640,
                height: 360,
            },
        );

        let mut value = serde_json::json!({
            "name": "Test",
            "sizes": ["small"]
        });

        resolve_sizes(&mut value, &size_map);

        let sizes = value["sizes"].as_array().unwrap();
        assert_eq!(sizes.len(), 1);
        assert_eq!(sizes[0]["name"], "small");
        assert_eq!(sizes[0]["width"], 640);
        assert_eq!(sizes[0]["height"], 360);
    }

    #[test]
    fn test_resolve_sizes_already_objects() {
        let size_map = HashMap::new();

        let mut value = serde_json::json!({
            "name": "Test",
            "sizes": [{"name": "custom", "width": 800, "height": 600}]
        });

        let original = value.clone();
        resolve_sizes(&mut value, &size_map);

        // Should be unchanged since sizes are already objects
        assert_eq!(value, original);
    }

    #[test]
    fn test_resolve_sizes_unknown_name() {
        let size_map = HashMap::new();

        let mut value = serde_json::json!({
            "name": "Test",
            "sizes": ["unknown"]
        });

        resolve_sizes(&mut value, &size_map);

        // Unknown size should remain as string
        let sizes = value["sizes"].as_array().unwrap();
        assert_eq!(sizes[0], "unknown");
    }

    #[test]
    fn test_parse_osnap_config() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("osnap.config.yaml");
        std::fs::write(
            &config_path,
            r#"
baseUrl: "http://localhost:3000"
snapshotDirectory: "../__image-snapshots__"
defaultSizes:
    - name: "small"
      width: 640
      height: 360
    - name: "xlarge"
      width: 1920
      height: 1080
"#,
        )
        .unwrap();

        let config = parse_osnap_config(&config_path).unwrap();

        assert_eq!(config.size_maps.by_name.len(), 2);
        assert_eq!(config.size_maps.by_name["small"].width, 640);
        assert_eq!(config.size_maps.by_dimensions[&(1920, 1080)], "xlarge");
        assert_eq!(
            config.snapshot_directory.as_deref(),
            Some("../__image-snapshots__")
        );
    }

    #[test]
    fn test_migrate_yaml_test_file_with_string_sizes() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("test.osnap.yaml");
        let target = dir.path().join("test.xsnap.json");

        std::fs::write(
            &source,
            r#"
- name: MyTest
  url: /test
  sizes:
    - small
    - xlarge
"#,
        )
        .unwrap();

        let mut size_map = HashMap::new();
        size_map.insert(
            "small".to_string(),
            SizeInfo {
                name: "small".to_string(),
                width: 640,
                height: 360,
            },
        );
        size_map.insert(
            "xlarge".to_string(),
            SizeInfo {
                name: "xlarge".to_string(),
                width: 1920,
                height: 1080,
            },
        );

        migrate_yaml_test_file(&source, &target, &size_map).unwrap();

        let result: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&target).unwrap()).unwrap();

        assert!(result.get("$schema").is_some());
        let tests = result["tests"].as_array().unwrap();
        assert_eq!(tests.len(), 1);

        let sizes = tests[0]["sizes"].as_array().unwrap();
        assert_eq!(sizes.len(), 2);
        assert_eq!(sizes[0]["name"], "small");
        assert_eq!(sizes[0]["width"], 640);
        assert_eq!(sizes[1]["name"], "xlarge");
        assert_eq!(sizes[1]["width"], 1920);
    }
}
