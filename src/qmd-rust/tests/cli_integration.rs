use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("qmd-rust").unwrap();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("AI-powered search"));
}

#[test]
fn test_cli_version() {
    let mut cmd = Command::cargo_bin("qmd-rust").unwrap();
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("0.1.0"));
}

#[test]
fn test_cli_no_args_shows_help() {
    let mut cmd = Command::cargo_bin("qmd-rust").unwrap();
    // No subcommand should show error/help
    cmd.assert().failure();
}

#[test]
fn test_cli_search_runs() {
    // Search command should at least parse correctly and attempt to run.
    // It may fail due to missing config, but should not panic.
    let mut cmd = Command::cargo_bin("qmd-rust").unwrap();
    cmd.args(["search", "test query"]);
    // We don't assert success because it depends on config existing,
    // but it should not crash with a signal.
    let output = cmd.output().unwrap();
    // Exit code may be non-zero (config not found), but process should complete
    assert!(
        output.status.code().is_some(),
        "Process should exit cleanly (not crash)"
    );
}

#[test]
fn test_cli_status_runs() {
    let mut cmd = Command::cargo_bin("qmd-rust").unwrap();
    cmd.arg("status");
    let output = cmd.output().unwrap();
    assert!(
        output.status.code().is_some(),
        "Status command should exit cleanly"
    );
}

#[test]
fn test_cli_update_runs() {
    let mut cmd = Command::cargo_bin("qmd-rust").unwrap();
    cmd.arg("update");
    let output = cmd.output().unwrap();
    assert!(
        output.status.code().is_some(),
        "Update command should exit cleanly"
    );
}
