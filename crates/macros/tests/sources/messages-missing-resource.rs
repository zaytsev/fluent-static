use fluent_static::message_bundle;
use fluent_static::value::Value;

#[message_bundle(
    resources=[
        ("tests/resources/simple-en.ftl", "en"),
        ("tests/resources/simple-it.ftl", "it")], 
    default_language = "en",
    functions = ("REVERSE" = reverse),
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
    let messages = Messages::default();
    assert_eq!("en foo", messages.test_param("foo"));
}
