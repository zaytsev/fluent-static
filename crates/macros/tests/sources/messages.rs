use fluent_static::value::Value;
use fluent_static::message_bundle;

#[message_bundle(
    resources = [
        ("tests/resources/simple-en.ftl", "en"),
        ("tests/resources/simple-fr.ftl", "fr")
    ], 
    functions = (
        "REVERSE" = reverse, 
    ),
    default_language = "en",
)
]
struct Messages;

impl Messages {
    fn reverse<'a, 'b>(
        positional_args: &'a [Value<'a>],
        _: &'a [(&'a str, Value<'a>)],
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

fn main() {
    let mut messages = Messages::default();
    messages.set_use_isolating(false);

    assert_eq!("en foo", messages.test_param("foo"));
    assert_eq!("en oof", messages.test_fn("foo"));
    assert_eq!("en 42", messages.number_fn(42));
}
