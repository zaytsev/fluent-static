# fluent-static


[![Latest version](https://img.shields.io/crates/v/fluent-static.svg)](https://crates.io/crates/fluent-static)

fluent-static provides simple to use, yet efficient way to add localization to Rust projects with [Fluent Localization System](https://projectfluent.org/).

fluent-static is inspired by and partially based on awesome [Fluent-rs](https://github.com/projectfluent/fluent-rs) project.

## Features

- **Compile-time Validation:** no chance to to make a typo in l10n message name or use it with the wrong number of arguments
- **Ergonomic API:** Just a method call `my_l10n.my_message()` to get l10n message
- **Minimal Runtime Overhead:** Fluent messages are translated into Rust code, no loading and parsing l10n resources at runtime required
- **Advanced Formatters:** Use (optionally) [Rust ICU bindings](https://github.com/google/rust_icu) to apply locale-specific formatting rules to currencies, measurement units values

## Usage

### Cargo dependencies

```toml
[dependencies]
fluent-static = "*"
```

### Create Fluent resource

```fluent
# <project root>/l10n/messages.ftl

say-hello = Hello, { $name }
    
```

### Declare message bundle

```rust
use fluent_static::message_bundle;

#[message_bundle(
    resources = [
        ("l10n/messages.ftl", "en"),
        // add more Fluent resources
        // ("i10n/errors.ftl", "en")
        // ("i10n/messages-fr.ftl", "fr")
    ],
    default_language = "en"
)]
pub struct Messages;
```

### Use the l10n messages

```rust
use fluent_static::MessageBundle;

pub fn main() {
    let lang = "en";
    let messages = Messagess::get(lang).unwrap_or_default();

    println!(messages.say_hello("World"));
}
    
```

### Notes

0. Language ID must be valid [Unicode Language Identifier](https://unicode.org/reports/tr35/tr35.html#unicode_language_id)
1. Message names are converted to *snake_case*
2. Function parameters are defined in the same exact order as they appear in a Fluent message defined in `default_language` bundle
3. Message must be defined for each supported language
4. Messages with arguments must have the same number and names of arguments (order doesn't matter) for each supported language
5. Messages and terms must be defined before they could be referenced

### A bit more advanced usage

* Use [codegen](/crates/codegen/README.md) in custom build scripts
* More customizations to [message_bundle](/crates/macros/README.md) proc macro for custom functions and formmaters

## Crate features

- **icu** enables different style of number formatting according to locale/language specific rules, requires native ICU libraries to be installed, see [example](/examples/simple/README.md)
- **axum** provides configurable value extractor to retrieve l10n bundle according to cookie or `Accept-Language` header value, see [example](/examples/axum/)
- **maud** adds support for Maud Rendere to l10n Message value, see [example](/examples/axum/)

## Contributing

Contributions are welcome! Please feel free to submit pull requests, report bugs, and suggest features via the issue tracker.

## License

This project is licensed under [MIT license](LICENSE.md). Feel free to use, modify, and distribute it as per the license conditions.

---
