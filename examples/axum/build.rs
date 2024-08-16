use fluent_static_codegen::{generate, MessageBundleCodeGenerator};

pub fn main() {
    generate!(
        "./l10n",
        MessageBundleCodeGenerator::new("en-US"),
        "l10n.rs"
    );
}
