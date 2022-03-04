use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_timezone_spelling() {
    let assert = Command::cargo_bin("rizzy")
        .unwrap()
        .arg("--zone")
        .arg("America/NewYork")
        .assert();

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    insta::assert_display_snapshot!(stderr);

    assert
        .failure()
        .stderr(predicate::str::contains("America/New_York"));
}

/// Make sure the examples from the README work.
#[test]
fn test_from_readme() {
    let assert = Command::cargo_bin("rizzy")
        .unwrap()
        .arg("--chi")
        .arg("tests/some.log.file")
        .assert();

    let assert = assert.success().stderr(predicate::eq(""));
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    insta::assert_display_snapshot!(stdout);
}
