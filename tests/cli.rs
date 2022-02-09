use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_timezone_spelling() {
    let assert = Command::cargo_bin("rizzy")
        .unwrap()
        .arg("--zone")
        .arg("America/NewYork")
        .assert();

    assert
        .failure()
        .stderr(predicate::str::contains("America/New_York"));
}
