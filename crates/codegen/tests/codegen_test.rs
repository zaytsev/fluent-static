use std::{env, fs, path::PathBuf};

use fluent_static_codegen::MessageBundleBuilder;

fn resources_base_dir() -> PathBuf {
    PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("'CARGO_MANIFEST_DIR' not set"))
        .join("tests/resources/")
}

fn output_dir() -> PathBuf {
    let out =
        PathBuf::from(env::var_os("OUT_DIR").expect("'OUT_DIR' not set")).join("generated/fluent");
    if !out.exists() {
        fs::create_dir_all(&out).unwrap();
    }
    out
}

#[test]
fn test_basic_messages() {
    let basic = MessageBundleBuilder::new("Basic")
        .with_default_language("en")
        .unwrap()
        .with_base_dir(resources_base_dir())
        .add_resource("en", "basic-en.ftl")
        .unwrap()
        .add_resource("it", "basic-it.ftl")
        .unwrap()
        .build()
        .unwrap();

    fs::write(output_dir().join("basic.rs"), basic.to_string())
        .expect("Error writing generated source");

    let test_cases = trybuild::TestCases::new();
    test_cases.pass("tests/sources/basic.rs");
}

#[test]
fn test_compound_messages() {
    let attributes = MessageBundleBuilder::new("Attributes")
        .with_default_language("en")
        .unwrap()
        .with_base_dir(resources_base_dir())
        .add_resource("en", "attributes-en.ftl")
        .unwrap()
        .add_resource("it", "attributes-it.ftl")
        .unwrap()
        .build()
        .unwrap();

    fs::write(output_dir().join("attributes.rs"), attributes.to_string())
        .expect("Error writing generated source");

    let test_cases = trybuild::TestCases::new();
    test_cases.pass("tests/sources/attributes.rs");
}

#[test]
fn test_selector_messages() {
    let strings = MessageBundleBuilder::new("Strings")
        .with_default_language("en")
        .unwrap()
        .with_base_dir(resources_base_dir())
        .add_resource("en", "selectors/strings-en.ftl")
        .unwrap()
        .add_resource("it", "selectors/strings-it.ftl")
        .unwrap()
        .build()
        .unwrap();

    fs::write(
        output_dir().join("selectors_strings.rs"),
        strings.to_string(),
    )
    .expect("Error writing generated source");

    let numbers = MessageBundleBuilder::new("Numbers")
        .with_default_language("en")
        .unwrap()
        .with_base_dir(resources_base_dir())
        .add_resource("en", "selectors/numbers-en.ftl")
        .unwrap()
        .add_resource("it", "selectors/numbers-it.ftl")
        .unwrap()
        .build()
        .unwrap();

    fs::write(
        output_dir().join("selectors_numbers.rs"),
        numbers.to_string(),
    )
    .expect("Error writing generated source");

    let plural_rules = MessageBundleBuilder::new("Prs")
        .with_default_language("en")
        .unwrap()
        .with_base_dir(resources_base_dir())
        .add_resource("en", "selectors/pluralrules-en.ftl")
        .unwrap()
        .add_resource("pl", "selectors/pluralrules-pl.ftl")
        .unwrap()
        .build()
        .unwrap();

    fs::write(
        output_dir().join("selectors_pluralrules.rs"),
        plural_rules.to_string(),
    )
    .expect("Error writing generated source");

    let test_cases = trybuild::TestCases::new();
    test_cases.pass("tests/sources/selectors/strings.rs");
    test_cases.pass("tests/sources/selectors/numbers.rs");
    test_cases.pass("tests/sources/selectors/pluralrules.rs");
}

#[test]
fn test_references() {
    let strings = MessageBundleBuilder::new("BasicRefs")
        .with_default_language("en")
        .unwrap()
        .with_base_dir(resources_base_dir())
        .add_resource("en", "refs/basic-en.ftl")
        .unwrap()
        .add_resource("it", "refs/basic-it.ftl")
        .unwrap()
        .build()
        .unwrap();

    fs::write(output_dir().join("basic_refs.rs"), strings.to_string())
        .expect("Error writing generated source");

    let test_cases = trybuild::TestCases::new();
    test_cases.pass("tests/sources/refs/basic.rs");
}

#[test]
fn test_functions() {
    let strings = MessageBundleBuilder::new("BuiltinFns")
        .with_default_language("en")
        .unwrap()
        .with_base_dir(resources_base_dir())
        .add_resource("en", "functions/builtins-en.ftl")
        .unwrap()
        .add_resource("it", "functions/builtins-it.ftl")
        .unwrap()
        .build()
        .unwrap();

    fs::write(output_dir().join("builtin_fns.rs"), strings.to_string())
        .expect("Error writing generated source");

    let test_cases = trybuild::TestCases::new();
    test_cases.pass("tests/sources/functions/builtins.rs");
}
