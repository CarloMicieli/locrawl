use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use assert_cmd::Command;

fn temp_path(name: &str) -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should be monotonic")
        .as_nanos();
    std::env::temp_dir().join(format!("locrawl-{}-{}", name, nanos))
}

#[tokio::test]
async fn test_validate_collection_rejects_broken_json() {
    let source_path = temp_path("invalid-collection.json");
    let invalid_payload = r#"{
        "name": "Broken collection",
        "railwayModels": []
    }"#;

    fs::write(&source_path, invalid_payload).expect("should write invalid payload");

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("locrawl"));
    cmd.args([
        "validate",
        "collection",
        "-s",
        source_path.to_string_lossy().as_ref(),
    ]);

    let output = cmd.output().expect("command should run");
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf8");

    assert!(!output.status.success());
    assert!(stdout.contains("Validation failed for Collection schema"));
    assert!(stdout.contains("Schema validation failed"));

    let _ = fs::remove_file(&source_path);
}

#[tokio::test]
async fn test_validate_track_accepts_mixed_case_schema_type() {
    let source_path = temp_path("valid-track.json");
    let valid_payload = r#"{
        "products": [],
        "inventories": []
    }"#;

    fs::write(&source_path, valid_payload).expect("should write valid payload");

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("locrawl"));
    cmd.args([
        "validate",
        "Track",
        "-s",
        source_path.to_string_lossy().as_ref(),
    ]);

    let output = cmd.output().expect("command should run");
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf8");

    assert!(output.status.success());
    assert!(stdout.contains("File is valid according to the Track schema"));

    let _ = fs::remove_file(&source_path);
}
