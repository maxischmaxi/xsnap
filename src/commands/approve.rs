use std::path::{Path, PathBuf};

use crate::config::global::load_global_config;

/// Options for the `xsnap approve` command.
pub struct ApproveOptions {
    pub config: String,
    pub all: bool,
    pub filter: Option<String>,
}

/// Run the approve command.
///
/// Moves updated screenshots from `__updated__` to `__base_images__`,
/// promoting them as the new baseline. In interactive mode, asks for
/// confirmation per file. With `--all`, approves everything.
pub fn run_approve(opts: ApproveOptions) -> anyhow::Result<()> {
    // 1. Load config to get the snapshot directory.
    let config_path = Path::new(&opts.config);
    let global = load_global_config(config_path)?;

    let updated_dir = PathBuf::from(&global.updated_directory);
    let base_dir = PathBuf::from(&global.base_directory);

    if !updated_dir.exists() {
        println!("No updated directory found. Nothing to approve.");
        return Ok(());
    }

    // Also clean up corresponding diff files.
    let diff_dir = PathBuf::from(&global.diff_directory);

    // 2. List files in updated directory, skipping .diff. files.
    let mut candidates: Vec<PathBuf> = Vec::new();
    for entry in std::fs::read_dir(&updated_dir)? {
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

        // Skip diff images (e.g., test-desktop-1920x1080-diff.png or test-desktop-1920x1080.diff.png)
        if filename.contains(".diff.") || filename.contains("-diff.") {
            continue;
        }

        // Skip non-PNG files.
        if !filename.ends_with(".png") {
            continue;
        }

        candidates.push(path);
    }

    // Sort for deterministic order.
    candidates.sort();

    // 3. Apply filter if present.
    if let Some(ref filter) = opts.filter {
        candidates.retain(|p| {
            let name = p
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            name.contains(filter.as_str())
        });
    }

    if candidates.is_empty() {
        println!("No updated snapshots to approve.");
        return Ok(());
    }

    println!("Found {} updated snapshot(s) to review:", candidates.len());

    // Ensure base directory exists.
    std::fs::create_dir_all(&base_dir)?;

    let mut approved_count = 0;
    let mut skipped_count = 0;

    // 4. Process each candidate.
    for candidate in &candidates {
        let filename = candidate
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let should_approve = if opts.all {
            true
        } else {
            // Interactive mode: ask user.
            let prompt = format!("Approve {}?", filename);
            dialoguer::Confirm::new()
                .with_prompt(&prompt)
                .default(false)
                .interact()
                .unwrap_or(false)
        };

        if should_approve {
            // Copy updated file to base_images.
            let target = base_dir.join(&filename);
            std::fs::copy(candidate, &target)?;
            println!("  Approved: {}", filename);
            approved_count += 1;

            // 5. Clean up: remove the approved file from updated directory.
            std::fs::remove_file(candidate)?;

            // Also remove the corresponding diff file if it exists.
            let diff_name_dash = filename.replace(".png", "-diff.png");
            let diff_path_dash = diff_dir.join(&diff_name_dash);
            if diff_path_dash.exists() {
                let _ = std::fs::remove_file(&diff_path_dash);
            }
        } else {
            println!("  Skipped: {}", filename);
            skipped_count += 1;
        }
    }

    println!(
        "\nDone: {} approved, {} skipped.",
        approved_count, skipped_count
    );
    Ok(())
}
