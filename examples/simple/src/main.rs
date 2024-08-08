fn main() {
    println!("{}", l10n::messages::hello("en", "World!").unwrap());
}

mod l10n {
    include!(concat!(env!("OUT_DIR"), "/generated/fluent/l10n.rs"));
}

#[cfg(test)]
mod test {
    #[test]
    fn test_l10n() {
        let actual = super::l10n::messages::hello("en", "foo").unwrap();
        assert_eq!("Hello,\nmy\ndear\nfried\n\u{2068}foo\u{2069}", actual);
    }

    #[test]
    fn test_attr() {
        let actual = super::l10n::messages::simple("en");
        assert_eq!("Simple", actual);
        let actual = super::l10n::messages::simple_attribute("en");
        assert_eq!("Simple Attribute", actual);
    }
}
