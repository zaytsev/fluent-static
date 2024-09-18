use fluent_static::message_bundle;

#[message_bundle(
    resources=[
        ("tests/resources/simple-en.ftl", "en"),
        ("tests/resources/simple-it.ftl", "it")], 
    default_language = "en")
]
struct Messages;

fn main() {
    let messages = Messages::default();
    assert_eq!("en foo", messages.test_param("foo"));
}
