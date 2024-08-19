use fluent_static::fluent_bundle::FluentValue;

fn main() {
    println!("{}", l10n::messages::hello("en", "World!").unwrap());
}

pub fn fluent_value_format<M>(value: &FluentValue, _: &M) -> Option<String> {
    if let FluentValue::String(s) = value {
        Some(format!("<{}>", s))
    } else {
        None
    }
}

mod l10n {
    fluent_static::include_source!("l10n.rs");
}

#[cfg(test)]
mod test {
    #[test]
    fn test_l10n() {
        let actual = super::l10n::messages::hello("en", "foo").unwrap();
        assert_eq!("Hello,\nmy\ndear\nfried\nfoo", actual);
    }

    #[test]
    fn test_attr() {
        let actual = super::l10n::messages::simple("en");
        assert_eq!("Simple", actual);
        let actual = super::l10n::messages::simple_attribute("en");
        assert_eq!("Simple Attribute", actual);
    }
}
