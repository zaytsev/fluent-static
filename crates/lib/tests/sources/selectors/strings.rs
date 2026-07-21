mod fluent {
    fluent_static::include_source!("selectors_strings.rs");
}

use fluent_static::MessageBundle;

fn main() {
    let mut bundle = fluent::Strings::get("it").unwrap();
    bundle.set_use_isolating(false);

    assert_eq!("IT Foo foo", bundle.sel_foo("foo", "bar", "baz"));
    assert_eq!("IT Bar foobar", bundle.sel_foo("bar", "foobar", "baz"));
    assert_eq!("IT Bar 123", bundle.sel_foo("42", "123", "bazbazbaz"));
    assert_eq!("IT Baz 2 Baz 1", bundle.sel_foo("baz", "1", "2"));

    let mut bundle = fluent::Strings::get("fr").unwrap_or_default();
    bundle.set_use_isolating(false);

    assert_eq!("EN Foo foo", bundle.sel_foo("foo", "bar", "baz"));
    assert_eq!("EN Bar foobar", bundle.sel_foo("bar", "foobar", "baz"));
    assert_eq!("EN Baz baz Baz 123", bundle.sel_foo("q", "123", "baz"));
    assert_eq!("EN Baz 2 Baz 1", bundle.sel_foo("baz", "1", "2"));
}
