mod fluent {
    fluent_static::include_source!("test_bundle.rs");
}

fn main() {
    let bundle = fluent::test::TestBundle::from(fluent_static::LanguageSpec::new("en".to_string()));

    assert_eq!("hello", bundle.hello());
    assert_eq!(
        "hello \u{2068}foo\u{2069}",
        bundle.hello_name("foo").unwrap()
    );
}
