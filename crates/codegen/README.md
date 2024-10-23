# fluent-static-codegen


[![Latest version](https://img.shields.io/crates/v/fluent-static-codegen.svg)](https://crates.io/crates/fluent-static-codegen)


Part of [fluent-static](/README.md) library providing simple to use, yet efficient way to add localization to Rust projects with [Fluent Localization System](https://projectfluent.org/).

fluent-static is inspired by and partially based on awesome [Fluent-rs](https://github.com/projectfluent/fluent-rs) project.

## Usage

### Cargo dependencies

Add `fluent-static-codegen` to project's `build-dependencies` section

```toml
[build-dependencies]
fluent-static-codegen = "*"
```

### Build script

The fluent-static-codegen requires [Cargo Build Script](https://doc.rust-lang.org/cargo/reference/build-scripts.html) to operate.

```rust


use std::{env, fs, path::PathBuf};

use fluent_static_codegen::MessageBundleBuilder;

fn resources_base_dir() -> PathBuf {
    PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("'CARGO_MANIFEST_DIR' not set"))
        .join("l10n")
}

fn output_dir() -> PathBuf {
    let out =
        PathBuf::from(env::var_os("OUT_DIR").expect("'OUT_DIR' not set")).join("generated/fluent");
    if !out.exists() {
        fs::create_dir_all(&out).unwrap();
    }
    out
}

pub fn main() {
    println!("cargo::rerun-if-changed=l10n/");

    let bundle = MessageBundleBuilder::new("Messages")
        .set_default_language("en")
        .expect("Default language should be valid language identifier")
        .set_resources_dir(resources_base_dir())
        .add_resource("en", "messages-en.ftl")
        .expect("Resource file should be valid Fluent resource")
        .add_resource("it", "messages-it.ftl")
        .expect("Resource file should be valid Fluent resource")
        .build()
        .unwrap();

    bundle
        .write_to_file(output_dir().join("messages.rs"))
        .expect("Output directory should exist and be writeable to save generated code");
}
  
```

### Registering Custom Fluent Functions

TBD

## License

This project is licensed under [MIT license](/LICENSE.md). Feel free to use, modify, and distribute it as per the license conditions.

---
