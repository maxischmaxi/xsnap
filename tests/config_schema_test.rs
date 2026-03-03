use xsnap::config::schema::{generate_schema, generate_test_schema};

#[test]
fn test_generate_schema_is_valid_json() {
    let schema_str = generate_schema();
    let parsed: serde_json::Value = serde_json::from_str(&schema_str).unwrap();

    // Root schema should be an object
    assert!(parsed.is_object(), "Schema should be a JSON object");

    // Should have a $schema meta-schema URI
    assert!(
        parsed.get("$schema").is_some(),
        "Schema should have a $schema field"
    );

    // Should have properties (since GlobalConfig is a struct with fields)
    let has_properties = parsed.get("properties").is_some()
        || parsed
            .get("$defs")
            .and_then(|d| d.as_object())
            .map(|d| d.values().any(|v| v.get("properties").is_some()))
            .unwrap_or(false);
    assert!(has_properties, "Schema should have properties somewhere");
}

#[test]
fn test_schema_has_required_fields() {
    let schema_str = generate_schema();
    let parsed: serde_json::Value = serde_json::from_str(&schema_str).unwrap();

    // The properties may be at the top level or under $defs depending on schemars
    // For a root schema, properties should be at top level
    let properties = parsed
        .get("properties")
        .expect("Root schema should have properties");

    let props_obj = properties.as_object().unwrap();

    // Verify that key GlobalConfig fields are present (camelCase due to serde rename)
    assert!(
        props_obj.contains_key("baseUrl"),
        "Should have baseUrl property, found: {:?}",
        props_obj.keys().collect::<Vec<_>>()
    );
    assert!(
        props_obj.contains_key("browser"),
        "Should have browser property"
    );
    assert!(
        props_obj.contains_key("defaultSizes"),
        "Should have defaultSizes property"
    );
    assert!(
        props_obj.contains_key("threshold"),
        "Should have threshold property"
    );
}

#[test]
fn test_generate_test_schema_is_valid_json() {
    let schema_str = generate_test_schema();
    let parsed: serde_json::Value = serde_json::from_str(&schema_str).unwrap();

    assert!(parsed.is_object(), "Test schema should be a JSON object");

    assert!(
        parsed.get("$schema").is_some(),
        "Test schema should have a $schema field"
    );

    let properties = parsed
        .get("properties")
        .expect("Test schema should have properties");

    let props_obj = properties.as_object().unwrap();

    assert!(
        props_obj.contains_key("$schema"),
        "Should have $schema property"
    );
    assert!(
        props_obj.contains_key("tests"),
        "Should have tests property"
    );
}
