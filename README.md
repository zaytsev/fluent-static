
# fluent-static

Fluent-static is a Rust library designed to generate statically typed function bindings from Fluent localization bundles. This allows for compile-time validation of localization message usage, enhancing both reliability and maintainability of your internationalized code base.

## Features

- **Compile-Time Validation:** Errors in localization message usage are caught at compile time, promoting reliability in multi-language projects.
- **Automatic Function Generation:** Converts Fluent messages into Rust functions dynamically, eliminating the need for manual updates when localization files change.
- **Easy Integration:** Works seamlessly within a standard Rust build environment with minimal configuration.

## Prerequisites

Before you begin, ensure you have the latest stable version of Rust installed on your machine. This project uses features that require Rust 2021 edition or later.

## Installation

Add the following to your `Cargo.toml` file:

```toml
[dependencies]
fluent-static = "*"

[build-dependencies]
fluent-static-codegen = "*"
```

## Usage

To integrate `fluent-static` into your Rust project, follow these steps:

### Step 1: Fluent Resources

Fluent resources should follow naming convention: `<resources_root>/<language_id>/<bundle_name>.ftl`, e.g. `l10n/en-US/messages.ftl`

### Step 2: Configure Code Generator

Create a `build.rs` file in your project root if it does not exist, and use the following template to generate Rust bindings for your Fluent resources:

```rust
use fluent_static_codegen::{generate, FunctionPerMessageCodeGenerator};
use std::{env, fs, path::Path};

pub fn main() {
    generate!("./l10n/", FunctionPerMessageCodeGenerator::new("en-US"), "l10n");
}
```

More details on code generation in `fluent-static-codegen` [README](fluent-static-codege/README.md)

### Step 3: Accessing Generated Functions

You can now use the generated functions in your main application or other modules:

```rust
fn main() {
    println!("{}", l10n::messages::hello("en", "World!").unwrap());
}

mod l10n {
    fluent_static::include_source!("l10n");
}
```

## Integrations

### Axum

Enable `axum` feature and use `fluent_static::axum::RequestLanguage` extractor in any Axum handler to access l10n messages with minimal boilerplate code

```toml
fluent-static = { version = "*", features = [ "axum" ] }
```

```rust
use axum::{routing::get, Router};
use fluent_static::axum::RequestLanguage;
use maud::{html, Markup};

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(hello_l10n));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn hello_l10n(RequestLanguage(msgs): RequestLanguage<l10n::messages::MessagesBundle>) -> String {
    let name = "Guest";
    format!("l10n: {}", msgs.hello(name).unwrap())
}

mod l10n {
    fluent_static::include_source!("l10n");
}
```

## Contributing

Contributions are welcome! Please feel free to submit pull requests, report bugs, and suggest features via the issue tracker.

## License

This project is licensed under [MIT license](LICENSE.md). Feel free to use, modify, and distribute it as per the license conditions.

---
