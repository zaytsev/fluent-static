mod fluent {
    use fluent_static::value::Value;

    pub fn fluent_uppercase<'a, 'b>(
        positional_args: &'a [Value<'a>],
        _named_args: &'a [(&'a str, Value<'a>)],
    ) -> Value<'b> {
        match positional_args.get(0) {
            Some(Value::String(s)) => Value::from(s.to_uppercase()),
            Some(Value::Number { value, format }) => Value::Number {
                value: value.clone(),
                format: format.clone(),
            },
            _ => Value::Empty,
        }
    }

    fluent_static::include_source!("custom_fns.rs");
}

use fluent_static::MessageBundle;

fn main() {
    let mut bundle = fluent::CustomFns::get("en").unwrap();
    bundle.set_use_isolating(false);

    assert_eq!("en FOO 10", bundle.uppercase_arg("foo", 10));

    let bundle = fluent::CustomFns::get("it").unwrap();
    assert_eq!(
        "it \u{2068}10\u{2069} \u{2068}FOO\u{2069}",
        bundle.uppercase_arg("foo", 10)
    );
}
