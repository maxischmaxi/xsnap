use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::config::global::load_global_config;
use crate::config::test::{discover_test_files, load_test_file};
use crate::config::types::{Size, TestConfig};
use crate::runner::executor::snapshot_filename;

/// Options for the `xsnap cleanup` command.
pub struct CleanupOptions {
    pub config: String,
}

/// Run the cleanup command.
///
/// Removes baseline snapshot images that no longer correspond to any
/// configured test + size combination. This helps keep the snapshot
/// directory free of orphaned images after tests are renamed or removed.
pub fn run_cleanup(opts: CleanupOptions) -> anyhow::Result<()> {
    // 1. Load config and discover all tests.
    let config_path = Path::new(&opts.config);
    let global = load_global_config(config_path)?;

    let base_dir = config_path.parent().unwrap_or_else(|| Path::new("."));
    let test_files = discover_test_files(base_dir, &global.test_pattern, &global.ignore_patterns)?;

    let mut all_tests: Vec<TestConfig> = Vec::new();

    for file in &test_files {
        let file_tests = load_test_file(file)?;
        all_tests.extend(file_tests);
    }

    // Include inline tests from the global config.
    all_tests.extend(global.tests.clone());

    // 2. Build set of expected snapshot filenames (test_name x size).
    let default_sizes = global.default_sizes.clone().unwrap_or_else(|| {
        vec![Size {
            name: "default".into(),
            width: 1280,
            height: 800,
        }]
    });

    let mut expected: HashSet<String> = HashSet::new();

    for test in &all_tests {
        let sizes = test.sizes.as_ref().unwrap_or(&default_sizes);
        for size in sizes {
            let filename = snapshot_filename(&test.name, size);
            expected.insert(filename);
        }
    }

    // 3. Scan base images directory.
    let base_images_dir = PathBuf::from(&global.base_directory);

    if !base_images_dir.exists() {
        println!("No base directory found. Nothing to clean up.");
        return Ok(());
    }

    let mut removed_count = 0;
    let mut kept_count = 0;

    for entry in std::fs::read_dir(&base_images_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let filename = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        // Only consider PNG files.
        if !filename.ends_with(".png") {
            continue;
        }

        // 4. Remove any files not in expected set.
        if !expected.contains(&filename) {
            std::fs::remove_file(&path)?;
            println!("  Removed orphaned snapshot: {}", filename);
            removed_count += 1;
        } else {
            kept_count += 1;
        }
    }

    println!(
        "\nCleanup complete: {} removed, {} kept.",
        removed_count, kept_count
    );

    Ok(())
}
