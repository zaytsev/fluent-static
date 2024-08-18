use fluent_static_codegen::{generate, MessageBundleCodeGenerator};

#[test]
fn test_message_bundle_struct() {
    generate!(
        "tests/resources/",
        MessageBundleCodeGenerator::new("en"),
        "test_bundle.rs"
    );

    let test_cases = trybuild::TestCases::new();
    test_cases.pass("tests/sources/test_bundle.rs");
}
