use self::common::run_test_cases;

mod common;

#[test]
fn new_project_test_cases() {
    run_test_cases("tests/new_project.md").unwrap();
}
