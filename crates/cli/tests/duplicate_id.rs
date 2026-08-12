use self::common::run_test_cases;

mod common;

#[test]
fn duplicate_id_test_cases() {
    run_test_cases("tests/duplicate_id.md").unwrap();
}
