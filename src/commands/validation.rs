use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use jsonschema::Validator;
use serde_json::Value;

use crate::commands::SchemaType;

pub(crate) fn manifest_schema_path() -> PathBuf {
    schema_dir().join("manifest_schema.json")
}

pub(crate) fn schema_path_for_type(schema_type: SchemaType) -> PathBuf {
    let file_name = match schema_type {
        SchemaType::Collection => "collection_schema.json",
        SchemaType::Wishlist => "wishlist_schema.json",
        SchemaType::DigitalRoster => "digital_roster_schema.json",
        SchemaType::Track => "track_import_schema.json",
        SchemaType::Manifest => "manifest_schema.json",
    };

    schema_dir().join(file_name)
}

pub(crate) fn load_schema(path: &Path) -> Result<Value> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read schema '{}'.", path.display()))?;

    let schema = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse schema '{}'.", path.display()))?;

    Ok(schema)
}

pub(crate) fn validate_value_with_schema(
    payload: &Value,
    schema_path: &Path,
    label: &str,
) -> Result<()> {
    let schema = load_schema(schema_path)?;
    let validator = jsonschema::options()
        .should_validate_formats(true)
        .build(&schema)
        .context("Failed to compile schema validator")?;

    validate_payload(&validator, payload, label)
}

pub(crate) fn validate_file(file_path: &Path, schema_path: &Path) -> Result<()> {
    let source_content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read source file '{}'.", file_path.display()))?;

    let source_json: Value = serde_json::from_str(&source_content)
        .with_context(|| format!("Failed to parse JSON from '{}'.", file_path.display()))?;

    validate_value_with_schema(&source_json, schema_path, &file_path.display().to_string())
}

fn validate_payload(validator: &Validator, payload: &Value, label: &str) -> Result<()> {
    let errors: Vec<String> = validator
        .iter_errors(payload)
        .map(|error| {
            let location = if error.instance_path.to_string().is_empty() {
                "$".to_string()
            } else {
                format!("${}", error.instance_path)
            };
            format!("{}: {}", location, error)
        })
        .collect();

    if errors.is_empty() {
        return Ok(());
    }

    bail!(
        "Schema validation failed for {}:\n{}",
        label,
        errors.join("\n")
    );
}

fn schema_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("schema")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_path(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be monotonic")
            .as_nanos();
        std::env::temp_dir().join(format!("locrawl-{}-{}", name, nanos))
    }

    #[test]
    fn validate_file_returns_schema_validation_error_for_broken_json() {
        let schema_path = temp_path("schema.json");
        let source_path = temp_path("source.json");

        let schema = r#"{
            "type": "object",
            "required": ["name"],
            "properties": {
                "name": { "type": "string" }
            }
        }"#;

        let source = r#"{"id":"missing-name"}"#;

        fs::write(&schema_path, schema).expect("should write test schema");
        fs::write(&source_path, source).expect("should write test source");

        let result = validate_file(&source_path, &schema_path);

        assert!(result.is_err());
        let error = result.expect_err("expected validation to fail");
        let error_text = error.to_string();
        assert!(error_text.contains("Schema validation failed"));
        assert!(error_text.contains("$"));

        let _ = fs::remove_file(&schema_path);
        let _ = fs::remove_file(&source_path);
    }

    // ------- schema_path_for_type -------

    #[test]
    fn schema_path_for_collection_ends_with_collection_schema_json() {
        let path = schema_path_for_type(crate::commands::SchemaType::Collection);
        assert!(path.to_string_lossy().ends_with("collection_schema.json"));
    }

    #[test]
    fn schema_path_for_wishlist_ends_with_wishlist_schema_json() {
        let path = schema_path_for_type(crate::commands::SchemaType::Wishlist);
        assert!(path.to_string_lossy().ends_with("wishlist_schema.json"));
    }

    #[test]
    fn schema_path_for_digital_roster_ends_with_digital_roster_schema_json() {
        let path = schema_path_for_type(crate::commands::SchemaType::DigitalRoster);
        assert!(
            path.to_string_lossy()
                .ends_with("digital_roster_schema.json")
        );
    }

    #[test]
    fn schema_path_for_track_ends_with_track_import_schema_json() {
        let path = schema_path_for_type(crate::commands::SchemaType::Track);
        assert!(path.to_string_lossy().ends_with("track_import_schema.json"));
    }

    #[test]
    fn schema_path_for_manifest_ends_with_manifest_schema_json() {
        let path = schema_path_for_type(crate::commands::SchemaType::Manifest);
        assert!(path.to_string_lossy().ends_with("manifest_schema.json"));
    }

    // ------- load_schema -------

    #[test]
    fn load_schema_successfully_reads_collection_schema() {
        let path = schema_path_for_type(crate::commands::SchemaType::Collection);
        let result = load_schema(&path);
        assert!(
            result.is_ok(),
            "should be able to load collection_schema.json"
        );
    }

    #[test]
    fn load_schema_returns_error_for_missing_file() {
        let path = std::path::Path::new("/nonexistent/path/schema.json");
        let result = load_schema(path);
        assert!(result.is_err());
    }

    // ------- validate_value_with_schema -------

    #[test]
    fn validate_value_with_schema_passes_for_valid_payload() {
        let schema_path = temp_path("valid-schema.json");
        let schema =
            r#"{"type":"object","required":["name"],"properties":{"name":{"type":"string"}}}"#;
        fs::write(&schema_path, schema).expect("write schema");

        let payload = serde_json::json!({ "name": "Märklin" });
        let result = validate_value_with_schema(&payload, &schema_path, "test");

        assert!(result.is_ok());
        let _ = fs::remove_file(&schema_path);
    }

    #[test]
    fn validate_value_with_schema_fails_for_wrong_type() {
        let schema_path = temp_path("type-schema.json");
        let schema =
            r#"{"type":"object","required":["count"],"properties":{"count":{"type":"integer"}}}"#;
        fs::write(&schema_path, schema).expect("write schema");

        let payload = serde_json::json!({ "count": "not-a-number" });
        let result = validate_value_with_schema(&payload, &schema_path, "test");

        assert!(result.is_err());
        let _ = fs::remove_file(&schema_path);
    }
}
