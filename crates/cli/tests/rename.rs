use self::common::run_test_cases;

mod common;

#[test]
fn rename_test_cases() {
    run_test_cases("tests/rename.md").unwrap();
}
