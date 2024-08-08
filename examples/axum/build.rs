use fluent_static_codegen::{generate, MessageBundleCodeGenerator};
use std::{env, fs, path::Path};

pub fn main() {
    generate!(
        "./l10n",
        MessageBundleCodeGenerator::new("en-US"),
        "l10n.rs"
    );
}
