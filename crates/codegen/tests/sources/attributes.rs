mod fluent {
    fluent_static::include_source!("attributes.rs");
}

use fluent_static::MessageBundle;

fn main() {
    let mut bundle = fluent::Attributes::get("it").unwrap();
    bundle.set_use_isolating(false);

    assert_eq!("ciao", bundle.hello());
    assert_eq!("ciao with attributes", bundle.hello_attr());
    assert_eq!("ciao foo", bundle.hello_name("foo"));
    assert_eq!("it without args", bundle.hello_name_no_args());
    assert_eq!(
        "second first",
        bundle.hello_name_with_args("second", "first")
    );
}
