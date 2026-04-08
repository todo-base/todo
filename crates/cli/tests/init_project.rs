use self::common::run_test_cases;

mod common;

#[test]
fn init_project_test_cases() {
    run_test_cases("tests/init_project.md").unwrap();
}
