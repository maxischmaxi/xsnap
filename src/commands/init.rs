use std::path::Path;

const DEFAULT_CONFIG: &str = r#"{
  "$schema": "https://raw.githubusercontent.com/maxischmaxi/xsnap/main/xsnap.schema.json",
  "baseUrl": "http://localhost:3000",
  "browser": {
    "version": "auto"
  },
  "fullScreen": true,
  "testPattern": "tests/**/*.xsnap.json",
  "ignorePatterns": ["node_modules"],
  "defaultSizes": [
    { "name": "desktop", "width": 1920, "height": 1080 },
    { "name": "tablet", "width": 768, "height": 1024 },
    { "name": "mobile", "width": 375, "height": 667 }
  ],
  "snapshotDirectory": "__snapshots__",
  "threshold": 0,
  "retry": 1,
  "diffPixelColor": { "r": 255, "g": 0, "b": 255 }
}
"#;

const EXAMPLE_TEST: &str = r#"{
  "$schema": "https://raw.githubusercontent.com/maxischmaxi/xsnap/main/xsnap.test.schema.json",
  "tests": [
    {
      "name": "example",
      "url": "/",
      "actions": [
        { "action": "wait", "timeout": 1000 }
      ]
    }
  ]
}
"#;

pub fn run_init() -> anyhow::Result<()> {
    let config_path = Path::new("xsnap.config.jsonc");
    if config_path.exists() {
        anyhow::bail!("xsnap.config.jsonc already exists");
    }
    std::fs::write(config_path, DEFAULT_CONFIG)?;
    println!("Created xsnap.config.jsonc");

    std::fs::create_dir_all("tests")?;
    let example_path = Path::new("tests/example.xsnap.json");
    if !example_path.exists() {
        std::fs::write(example_path, EXAMPLE_TEST)?;
        println!("Created tests/example.xsnap.json");
    }

    std::fs::create_dir_all("__snapshots__/__base_images__")?;
    std::fs::create_dir_all("__snapshots__/__updated__")?;
    std::fs::create_dir_all("__snapshots__/__current__")?;
    println!("Created __snapshots__/ directory structure");
    println!("\nxsnap initialized! Edit xsnap.config.jsonc to get started.");
    Ok(())
}
