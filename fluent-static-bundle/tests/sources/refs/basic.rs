mod fluent {
    fluent_static::include_source!("basic_refs.rs");
}

use fluent_static::MessageBundle;

fn main() {
    let bundle = fluent::BasicRefs::get("en").unwrap();

    assert_eq!("hello", bundle.hello());
    assert_eq!("en: hello foo", bundle.hello_name("foo"));
    assert_eq!(
        "arg1 hello foobar",
        bundle.term_reference_with_named_param("arg1")
    );
    assert_eq!("en hello 1", bundle.two_term_refs());
    assert_eq!("en nested hello bar", bundle.nested_term_ref());

    assert_eq!("en en", bundle.term_with_attrs());

    assert_eq!("en hello hello", bundle.message_as_arg_value());
}
