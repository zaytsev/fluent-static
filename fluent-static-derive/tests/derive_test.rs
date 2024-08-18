#[test]
fn test_derive_fluent_bundle() {
    std::env::set_var(
        "CARGO_MANIFEST_DIR_OVERRIDE",
        std::env::var_os("CARGO_MANIFEST_DIR").unwrap(),
    );

    let test_cases = trybuild::TestCases::new();
    test_cases.pass("tests/derive_basic.rs");
}
