use self::common::run_test_cases;

mod common;

#[test]
fn add_issue_test_cases() {
    run_test_cases("tests/add_issue.md").unwrap();
}
