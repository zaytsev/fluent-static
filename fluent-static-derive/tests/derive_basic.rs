use fluent_static_derive::FluentBundle;

#[derive(FluentBundle)]
#[fluent("tests/resources/simple-*.ftl", default_language = "en")]
struct Messages;

fn main() {}
