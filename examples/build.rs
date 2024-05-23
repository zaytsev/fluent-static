use std::{env, fs, path::Path};

pub fn main() {
    let src = fluent_static_generate::build::generate("./l10n/", "en_US")
        .expect("Error generating fluent message bindings");

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let destination = Path::new(&out_dir).join("l10n.rs");

    fs::write(destination, src).expect("Error writing generated sources");
}
