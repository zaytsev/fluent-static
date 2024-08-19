use fluent_static_codegen::{generate, FluentBundleOptions, FunctionPerMessageCodeGenerator};

pub fn main() {
    generate!(
        "./l10n",
        FunctionPerMessageCodeGenerator::new_with_options(
            "en-US",
            FluentBundleOptions {
                use_isolating: false,
                transform_fn: None,
                format_fn: None,
                // format_fn: Some("crate::fluent_value_format".to_string()),
            }
        ),
        "l10n.rs"
    );
}
