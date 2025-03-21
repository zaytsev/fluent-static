use fluent_static::value::number::format::{CurrencyCode, NumberFormat};
use fluent_static::value::Value;

mod l10n {
    use fluent_static::message_bundle;
    #[message_bundle(
        resources = [
            ("l10n/en-US/messages.ftl", "en-US"), 
            ("l10n/fr-CH/messages.ftl", "fr-CH"), 
        ],
        default_language = "en-US")]
    pub struct Messages;
}

fn main() {
    let messages = l10n::Messages::default();
    println!("{}", messages.hello("world"));

    println!(
        "{}",
        messages.total_amount(Value::formatted_number(
            42.0,
            NumberFormat::currency(CurrencyCode::USD)
        ))
    )
}
