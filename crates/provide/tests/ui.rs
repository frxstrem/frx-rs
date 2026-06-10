#![allow(missing_docs)]

#[cfg(rust_analyzer)]
mod ui {
    mod fail {
        mod extending_lifetime;
    }
}

#[test]
fn ui() {
    let test_cases = trybuild::TestCases::new();
    test_cases.compile_fail("tests/ui/fail/*.rs")
}
