mod fluent {
    fluent_static::include_source!("basic.rs");
}

use fluent_static::MessageBundle;

fn main() {
    let bundle = fluent::Basic::get("en").unwrap();

    assert_eq!("hello", bundle.hello());
    assert_eq!("hello foo", bundle.hello_name("foo"));
}
