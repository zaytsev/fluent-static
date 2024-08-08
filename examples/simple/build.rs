use fluent_static_codegen::{generate, FunctionPerMessageCodeGenerator};
use std::{env, fs, path::Path};

pub fn main() {
    generate!(
        "./l10n",
        FunctionPerMessageCodeGenerator::new("en-US"),
        "l10n.rs"
    );
}
