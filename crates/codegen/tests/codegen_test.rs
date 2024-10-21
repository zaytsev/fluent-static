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
        .set_default_language("en")
        .unwrap()
        .set_resources_dir(resources_base_dir())
        .add_resource("en", "basic-en.ftl")
        .unwrap()
        .add_resource("it", "basic-it.ftl")
        .unwrap()
        .build()
        .unwrap();

    basic
        .write_to_file(output_dir().join("basic.rs"))
        .expect("Error writing generated source");

    let test_cases = trybuild::TestCases::new();
    test_cases.pass("tests/sources/basic.rs");
}

#[test]
fn test_compound_messages() {
    let attributes = MessageBundleBuilder::new("Attributes")
        .set_default_language("en")
        .unwrap()
        .set_resources_dir(resources_base_dir())
        .add_resource("en", "attributes-en.ftl")
        .unwrap()
        .add_resource("it", "attributes-it.ftl")
        .unwrap()
        .build()
        .unwrap();

    attributes
        .write_to_file(output_dir().join("attributes.rs"))
        .expect("Error writing generated source");

    let test_cases = trybuild::TestCases::new();
    test_cases.pass("tests/sources/attributes.rs");
}

#[test]
fn test_selector_messages() {
    let strings = MessageBundleBuilder::new("Strings")
        .set_default_language("en")
        .unwrap()
        .set_resources_dir(resources_base_dir())
        .add_resource("en", "selectors/strings-en.ftl")
        .unwrap()
        .add_resource("it", "selectors/strings-it.ftl")
        .unwrap()
        .build()
        .unwrap();

    strings
        .write_to_file(output_dir().join("selectors_strings.rs"))
        .expect("Error writing generated source");

    let numbers = MessageBundleBuilder::new("Numbers")
        .set_default_language("en")
        .unwrap()
        .set_resources_dir(resources_base_dir())
        .add_resource("en", "selectors/numbers-en.ftl")
        .unwrap()
        .add_resource("it", "selectors/numbers-it.ftl")
        .unwrap()
        .build()
        .unwrap();

    numbers
        .write_to_file(output_dir().join("selectors_numbers.rs"))
        .expect("Error writing generated source");

    let plural_rules = MessageBundleBuilder::new("Prs")
        .set_default_language("en")
        .unwrap()
        .set_resources_dir(resources_base_dir())
        .add_resource("en", "selectors/pluralrules-en.ftl")
        .unwrap()
        .add_resource("pl", "selectors/pluralrules-pl.ftl")
        .unwrap()
        .build()
        .unwrap();

    plural_rules
        .write_to_file(output_dir().join("selectors_pluralrules.rs"))
        .expect("Error writing generated source");

    let test_cases = trybuild::TestCases::new();
    test_cases.pass("tests/sources/selectors/strings.rs");
    test_cases.pass("tests/sources/selectors/numbers.rs");
    test_cases.pass("tests/sources/selectors/pluralrules.rs");
}

#[test]
fn test_references() {
    let bundle = MessageBundleBuilder::new("BasicRefs")
        .set_default_language("en")
        .unwrap()
        .set_resources_dir(resources_base_dir())
        .add_resource("en", "refs/basic-en.ftl")
        .unwrap()
        .add_resource("it", "refs/basic-it.ftl")
        .unwrap()
        .build()
        .unwrap();

    bundle
        .write_to_file(output_dir().join("basic_refs.rs"))
        .expect("Error writing generated source");

    let test_cases = trybuild::TestCases::new();
    test_cases.pass("tests/sources/refs/basic.rs");
}

#[test]
fn test_functions() {
    let bundle = MessageBundleBuilder::new("BuiltinFns")
        .set_default_language("en")
        .unwrap()
        .set_resources_dir(resources_base_dir())
        .add_resource("en", "functions/builtins-en.ftl")
        .unwrap()
        .add_resource("it", "functions/builtins-it.ftl")
        .unwrap()
        .build()
        .unwrap();

    bundle
        .write_to_file(output_dir().join("builtin_fns.rs"))
        .expect("Error writing generated source");

    let test_cases = trybuild::TestCases::new();
    test_cases.pass("tests/sources/functions/builtins.rs");
}
