fn main() {
    println!("{}", l10n::messages::hello("en", "World!"));
}

mod l10n {
    include!(concat!(env!("OUT_DIR"), "/l10n.rs"));
}

#[cfg(test)]
mod test {
    #[test]
    fn test_l10n() {
        let actual = super::l10n::messages::hello("en", "foo");
        assert_eq!("Hello, \u{2068}foo\u{2069}", actual);
    }
}
