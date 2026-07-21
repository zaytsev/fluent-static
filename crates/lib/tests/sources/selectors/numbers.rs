mod fluent {
    fluent_static::include_source!("selectors_numbers.rs");
}

use fluent_static::MessageBundle;

fn main() {
    let mut bundle = fluent::Numbers::get("en").unwrap();
    bundle.set_use_isolating(false);

    assert_eq!("minus two", bundle.test_numbers(-2));
    assert_eq!("zero", bundle.test_numbers(0i32));
    assert_eq!("zero", bundle.test_numbers(0f64));
    assert_eq!("one.o", bundle.test_numbers(1));
    assert_eq!("one.o", bundle.test_numbers(1.0));
    assert_eq!("other", bundle.test_numbers(1.000001));
    assert_eq!("the answer", bundle.test_numbers(42.0));
    assert_eq!("other", bundle.test_numbers(-9999));
}
