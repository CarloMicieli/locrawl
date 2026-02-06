use assert_cmd::Command;

#[test]
fn test_locrawl_info_command() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("locrawl"));
    cmd.arg("info");

    // This will fail until the command is implemented
    let assert = cmd.assert();
    assert.success();
}

#[test]
fn test_locrawl_info_output() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("locrawl"));
    cmd.arg("info");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("locrawl"));
    assert!(stdout.contains("v0.1.0")); // version
    assert!(stdout.contains("CLI tool")); // summary
}
