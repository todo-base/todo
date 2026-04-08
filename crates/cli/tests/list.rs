use self::common::run_test_cases;

mod common;

#[test]
fn list_test_cases() {
    run_test_cases("tests/list.md").unwrap();
}

#[test]
fn list_complex_test_cases() {
    run_test_cases("tests/list_complex.md").unwrap();
}

#[test]
fn list_display_test_cases() {
    run_test_cases("tests/list_display.md").unwrap();
}
