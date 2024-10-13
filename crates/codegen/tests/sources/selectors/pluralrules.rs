mod fluent {
    fluent_static::include_source!("selectors_pluralrules.rs");
}

use fluent_static::MessageBundle;

fn main() {
    let bundle = fluent::Prs::get("en").unwrap();

    assert_eq!(
        "Foo added a new photo to his stream.",
        bundle.shared_photos("Foo", 1, "male")
    );

    assert_eq!(
        "Jane added 55 new photos to her stream.",
        bundle.shared_photos("Jane", 55, "female")
    );

    assert_eq!(
        "Foobar added 666 new photos to their stream.",
        bundle.shared_photos("Foobar", 666, "baz")
    );

    let bundle = fluent::Prs::get("pl").unwrap();

    // Male gender
    assert_eq!(
        "Foo dodał nowe zdjęcie do swojego strumienia.",
        bundle.shared_photos("Foo", 1, "male")
    );
    assert_eq!(
        "Foo dodał 2 nowe zdjęcia do swojego strumienia.",
        bundle.shared_photos("Foo", 2, "male")
    );
    assert_eq!(
        "Foo dodał 3 nowe zdjęcia do swojego strumienia.",
        bundle.shared_photos("Foo", 3, "male")
    );
    assert_eq!(
        "Foo dodał 5 nowych zdjęć do swojego strumienia.",
        bundle.shared_photos("Foo", 5, "male")
    );
    assert_eq!(
        "Foo dodał 22 nowe zdjęcia do swojego strumienia.",
        bundle.shared_photos("Foo", 22, "male")
    );
    assert_eq!(
        "Foo dodał 25 nowych zdjęć do swojego strumienia.",
        bundle.shared_photos("Foo", 25, "male")
    );

    // Female gender
    assert_eq!(
        "Bar dodała nowe zdjęcie do swojego strumienia.",
        bundle.shared_photos("Bar", 1, "female")
    );
    assert_eq!(
        "Bar dodała 2 nowe zdjęcia do swojego strumienia.",
        bundle.shared_photos("Bar", 2, "female")
    );
    assert_eq!(
        "Bar dodała 4 nowe zdjęcia do swojego strumienia.",
        bundle.shared_photos("Bar", 4, "female")
    );
    assert_eq!(
        "Bar dodała 5 nowych zdjęć do swojego strumienia.",
        bundle.shared_photos("Bar", 5, "female")
    );
    assert_eq!(
        "Bar dodała 23 nowe zdjęcia do swojego strumienia.",
        bundle.shared_photos("Bar", 23, "female")
    );
    assert_eq!(
        "Bar dodała 26 nowych zdjęć do swojego strumienia.",
        bundle.shared_photos("Bar", 26, "female")
    );

    // Other/unknown gender
    assert_eq!(
        "Baz dodał(a) nowe zdjęcie do swojego strumienia.",
        bundle.shared_photos("Baz", 1, "other")
    );
    assert_eq!(
        "Baz dodał(a) 2 nowe zdjęcia do swojego strumienia.",
        bundle.shared_photos("Baz", 2, "other")
    );
    assert_eq!(
        "Baz dodał(a) 4 nowe zdjęcia do swojego strumienia.",
        bundle.shared_photos("Baz", 4, "other")
    );
    assert_eq!(
        "Baz dodał(a) 5 nowych zdjęć do swojego strumienia.",
        bundle.shared_photos("Baz", 5, "other")
    );
    assert_eq!(
        "Baz dodał(a) 24 nowe zdjęcia do swojego strumienia.",
        bundle.shared_photos("Baz", 24, "other")
    );
    assert_eq!(
        "Baz dodał(a) 27 nowych zdjęć do swojego strumienia.",
        bundle.shared_photos("Baz", 27, "other")
    );
}
