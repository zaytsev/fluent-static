mod fluent {
    fluent_static::include_source!("builtin_fns.rs");
}

use fluent_static::MessageBundle;

fn main() {
    let bundle = fluent::BuiltinFns::get("en").unwrap();

    assert_eq!("en 10", bundle.simple_number(10));
    assert_eq!("en 11", bundle.simple_number("11"));
    assert_eq!("en 4242", bundle.number_const());
    assert_eq!("en 42", bundle.number_with_named_arg(42));
    assert_eq!("en 100", bundle.number_msg_ref());
    assert_eq!("en term 111", bundle.number_term_ref());
    assert_eq!("en zero", bundle.selector_number(0));
    assert_eq!("en other", bundle.selector_number(0.01));
    assert_eq!("en inception 0.01", bundle.number_number(0.01));
    assert_eq!("en msg en 0.01", bundle.term_arg_msg(0.01));
}
