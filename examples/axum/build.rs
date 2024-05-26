use fluent_static_codegen::{generate, FunctionPerMessageCodeGenerator};
use std::{env, fs, path::Path};

pub fn main() {
    let src = generate("./l10n/", FunctionPerMessageCodeGenerator::new("en-US"))
        .expect("Error generating fluent message bindings");

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let destination = Path::new(&out_dir).join("l10n.rs");

    fs::write(destination, src).expect("Error writing generated sources");
}
