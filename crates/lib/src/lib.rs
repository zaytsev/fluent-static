pub use intl_pluralrules;
pub use once_cell;
pub use unic_langid;

pub use fluent_static_function as function;
pub use fluent_static_macros::message_bundle;
pub use fluent_static_value as value;

mod message;

pub use message::Message;
pub mod formatter;

pub mod support;

#[macro_export]
macro_rules! include_source {
    ($name:expr) => {
        include!(concat!(env!("OUT_DIR"), "/generated/fluent/", $name));
    };
}

pub trait LanguageAware {
    fn language_id(&self) -> &str;
}

pub trait MessageBundle: LanguageAware + Default {
    fn get(language_id: &str) -> Option<Self>
    where
        Self: Sized;
    fn default_language_id() -> &'static str;
    fn supported_language_ids() -> &'static [&'static str];
}
