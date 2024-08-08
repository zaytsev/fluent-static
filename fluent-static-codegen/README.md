# fluent-static-codegen

Part of [fluent-static](../README.md) library responsible for generating Rust code from [Fluent](https://projectfluent.org/) resources.

The project is heavily relying on [fluent-rs](https://github.com/projectfluent/fluent-rs) for parsing and processing Fluent resources.

## Usage

### Fluent resources

The code generator requires that Fluent resources adhere to the naming convention: `<l10n_root>/<language_id>/<bundle_name>`.

* `language_id` must be a valid [Unicode Language Identifier](https://unicode.org/reports/tr35/tr35.html#unicode_language_id)
* `bundle_name` must be a valid Rust identifier

### Cargo dependencies

Add `fluent-static-codegen` to project's `build-dependencies` section

```toml
[build-dependencies]
fluent-static-codegen = "*"
```

### Build script

The fluent-static-codegen requires [Cargo Build Script](https://doc.rust-lang.org/cargo/reference/build-scripts.html) to operate.

```rust

use fluent_static_codegen::{generate, FunctionPerMessageCodeGenerator};
use std::{env, fs, path::Path};

pub fn main() {
    // use <project_root>/l10n as base path to search for Fluent resources
    // generate rust function per each Fluent message
    // use "en-US" Language ID as the default language
    let src = generate("./l10n/", FunctionPerMessageCodeGenerator::new("en-US"))
        .expect("Error generating message bindings");

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let destination = Path::new(&out_dir).join("l10n.rs");

    fs::write(destination, src).expect("Error writing generated sources");
}
  
```

### Code generation

#### Generated code naming conventions

* Fluent message names converted to Rust snake_cased identifiers
  * `my-message` -> `my_message`
  * `msgTest` -> `msg_test`
  * `msg1` -> `msg_1`
* Fluent message variables mapped to snake_cased indentifiers also
* Fluent message attributes mapped to `<message_name>_<attribute_name>`
  * `my-button.title` -> `my_button_title`

#### Fluent message mapped to a function

The `fluent_static_codegen::FunctionPerMessageCodeGenerator` generates module per each `message_bundle` and maps messages to a functions 

```rust
pub mod <bundle_name> {
    use fluent_static::fluent_bundle::{FluentValue, FluentError};
    use fluent_static::Message;

    pub fn <fluent_message_no_args>(language_id: impl AsRef<str>) -> Message {
    }

    pub fn <fluent_message_with_args>(language_id: impl AsRef<str>, var1: impl Into<FluentValue>>, ...) -> Result<Message, FluentError> {
    }
}
```

#### Fluent message mapped to a method

`fluent_static_codegen::FunctionPerMessageCodeGenerator` generates struct per each `message_bundle` 
and maps messages to the struct's public methods 

```rust
pub mod <bundle_name>
    use fluent_static::fluent_bundle::{FluentValue, FluentError};
    use fluent_static::Message;

    pub struct <BundleName>Bundle {
    }

    impl <BundleName>Bundle {
        pub fn all_languages() -> &'static [&'static str] {
        }

        pub fn current_language(&self) -> &LanguageSpec {
        }

        pub fn <fluent_message_no_args>(&self) -> Message {
        }

        pub fn <fluent_message_some_args>(&self, name: impl Into<FluentValue>, ...) -> Result<Message, FluentError> {
        }
    }
}  
```

### Compile-time validation

The following conditions are validated at compilation time to ensure that each message in a `bundle_name` is correctly defined:

1. The message must be defined across all supported languages.
2. All instances of the message must share the identical set of variables (if any variables exist).
3. Message attributes must remain consistent across all languages.

A compile-time error will be raised if any of these conditions are not met, guaranteeing consistency and correctness across different language implementations.

## Contributing

Contributions are welcome! Please feel free to submit pull requests, report bugs, and suggest features via the issue tracker.

## License

This project is licensed under [MIT license](LICENSE.md). Feel free to use, modify, and distribute it as per the license conditions.

---
