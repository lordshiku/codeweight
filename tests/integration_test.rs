use std::fs;
use std::path::PathBuf;

use assert_cmd::Command;
use codeweight::analyzer::{analyze_content, Language};
use predicates::prelude::*;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn read_fixture(name: &str) -> String {
    fs::read_to_string(fixture_path(name)).expect("fixture should be readable")
}

#[test]
fn python_fixture_metrics() {
    let content = read_fixture("sample.py");
    let metrics = analyze_content(Language::Python, &content).expect("analysis should succeed");

    assert_eq!(metrics.total_lines, 16);
    assert_eq!(metrics.blank_lines, 4);
    assert_eq!(metrics.comment_lines, 1);
    assert_eq!(metrics.code_lines, 11);
    assert_eq!(metrics.function_count, 2);
    assert_eq!(metrics.cyclomatic_complexity, 6);
    assert_eq!(metrics.max_function_complexity, 4);
    assert!(metrics.max_nesting_depth >= 2);
    assert!(metrics.maintainability_score > 50.0);
}

#[test]
fn javascript_fixture_metrics() {
    let content = read_fixture("sample.js");
    let metrics =
        analyze_content(Language::JavaScript, &content).expect("analysis should succeed");

    assert_eq!(metrics.function_count, 3, "add, classify, and identity");
    assert!(metrics.cyclomatic_complexity >= 6);
    assert!(metrics.code_lines >= 15);
    assert!(metrics.max_nesting_depth >= 2);
}

#[test]
fn rust_fixture_metrics() {
    let content = read_fixture("sample.rs");
    let metrics = analyze_content(Language::Rust, &content).expect("analysis should succeed");

    assert_eq!(metrics.function_count, 2);
    assert_eq!(metrics.cyclomatic_complexity, 6);
    assert!(metrics.max_nesting_depth >= 2);
}

#[test]
fn java_fixture_metrics() {
    let content = read_fixture("sample.java");
    let metrics = analyze_content(Language::Java, &content).expect("analysis should succeed");

    assert_eq!(metrics.function_count, 2);
    assert!(metrics.cyclomatic_complexity >= 5);
    assert!(metrics.max_nesting_depth >= 2);
}

#[test]
fn cli_analyze_fixtures_directory() {
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures");

    Command::cargo_bin("codeweight")
        .expect("binary should exist")
        .args(["analyze", fixtures.to_str().expect("utf-8 path")])
        .assert()
        .success()
        .stdout(predicate::str::contains("sample.py"))
        .stdout(predicate::str::contains("Summary"));
}

#[test]
fn cli_summary_json_output() {
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures");

    Command::cargo_bin("codeweight")
        .expect("binary should exist")
        .args([
            "summary",
            fixtures.to_str().expect("utf-8 path"),
            "--json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"file_count\""))
        .stdout(predicate::str::contains("\"avg_maintainability_score\""));
}

#[test]
fn cli_missing_path_exits_with_error() {
    Command::cargo_bin("codeweight")
        .expect("binary should exist")
        .args(["analyze", "definitely/not/a/real/path"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("path does not exist"));
}
