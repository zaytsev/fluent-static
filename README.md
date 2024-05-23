
# fluent-static

Fluent-static is a Rust library designed to generate statically typed function bindings from Fluent localization bundles. This allows for compile-time validation of localization message usage, enhancing both reliability and maintainability of your internationalized code base.

## Features

- **Compile-Time Validation:** Errors in localization message usage are caught at compile time, promoting reliability in multi-language projects.
- **Automatic Function Generation:** Converts Fluent messages into Rust functions dynamically, eliminating the need for manual updates when localization files change.
- **Easy Integration:** Works seamlessly within a standard Rust build environment with minimal configuration.

## Prerequisites

Before you begin, ensure you have the latest stable version of Rust installed on your machine. This project uses features that require Rust 2018 edition or later.

## Installation

Add the following to your `Cargo.toml` file:

```toml
[dependencies]
fluent-static = "0.1.0"  # Make sure to use the latest version

[build-dependencies]
fluent-static-codegen = "0.1.0"
```

## Usage

To integrate `fluent-static` into your Rust project, follow these steps:

### Step 1: Fluent Resources

Ensure your Fluent resources are placed under a specific directory, e.g., `./l10n/`, and include at least one localization, e.g., `en_US.ftl`.

### Step 2: Build Script Setup

Create a `build.rs` file in your project root if it does not exist, and use the following template to generate Rust bindings for your Fluent files:

```rust
use std::{env, fs, path::Path};

pub fn main() {
    let src = fluent_static_codegen::build::generate("./l10n/", "en_US")
        .expect("Error generating fluent message bindings");

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let destination = Path::new(&out_dir).join("l10n.rs");

    fs::write(destination, src).expect("Error writing generated sources");
}
```

### Step 3: Accessing Generated Functions

You can now use the generated functions in your main application or other modules:

```rust
fn main() {
    println!("{}", l10n::messages::hello("en", "World!"));
}

mod l10n {
    include!(concat!(env!("OUT_DIR"), "/l10n.rs"));
}
```

### Testing

```rust
#[cfg(test)]
mod test {
    #[test]
    fn test_l10n() {
        let actual = super::l10n::messages::hello("en", "foo");
        assert_eq!("Hello, \u{2068}foo\u{2069}", actual);
    }
}
```

The tests assure that the localization functions interact correctly with inputs and match expected outputs, using the generated Rust functions from Fluent files.

## Contributing

Contributions are welcome! Please feel free to submit pull requests, report bugs, and suggest features via the issue tracker.

## License

This project is licensed under [MIT license](LICENSE.md). Feel free to use, modify, and distribute it as per the license conditions.

---
