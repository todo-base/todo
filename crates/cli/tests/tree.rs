use self::common::run_test_cases;

mod common;

#[test]
fn tree_test_cases() {
    run_test_cases("tests/tree.md").unwrap();
}

#[test]
fn tree_complex_test_cases() {
    run_test_cases("tests/tree_complex.md").unwrap();
}
