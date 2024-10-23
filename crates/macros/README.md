# fluent-static-macros


[![Latest version](https://img.shields.io/crates/v/fluent-static-macros.svg)](https://crates.io/crates/fluent-static-macros)


Part of [fluent-static](/README.md) library providing simple to use, yet efficient way to add localization to Rust projects with [Fluent Localization System](https://projectfluent.org/).

fluent-static is inspired by and partially based on awesome [Fluent-rs](https://github.com/projectfluent/fluent-rs) project.

## Usage example

```rust
use fluent_static::value::Value;
use fluent_static::message_bundle;

#[message_bundle(
    resources = [
        ("l10n/simple-en.ftl", "en"),
        ("l10n/extra-en.ftl", "en"),
    ], 
    default_language = "en",
    // Optional mapping of custom Fluent functions to Rust implementations
    functions = (
        "REVERSE" = reverse, // 'REVERSE' is mapped to Self::reverse function 
        // more custom functions
    ),
    // Optional custom value formatter function
    formatter = "custom_formatter"
)]
struct Messages;
    
fn custom_formatter(language_id: &str, value: &Value, out: &mut impl std::fmt::Write) -> std::fmt::Result {
    out.write_char('|')?;
    match value {
        Value::String(s) => out.write_str(s),
        Value::Number{ value, ..} => write!(out, "{:?}", value),
        _ => Ok(())
    }?;
    out.write_char('|')
}

impl Messages {
    fn reverse<'a, 'b>(
        positional_args: &'a [Value<'a>],
        named_args: &'a [(&'a str, Value<'a>)],
    ) -> Value<'b> {
        if let Some(Value::String(s)) = positional_args.get(0) {
            // not unicode-proof
            let reversed = s.chars().rev().collect::<String>();
            Value::from(reversed)
        } else {
            Value::Error
        }
    }
}

```

## License

This project is licensed under [MIT license](/LICENSE.md). Feel free to use, modify, and distribute it as per the license conditions.

---
