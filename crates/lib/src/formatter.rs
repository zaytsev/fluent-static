#[cfg(not(feature = "icu"))]
use crate::value::Number;
#[cfg(not(feature = "icu"))]
use crate::value::Value;

#[cfg(feature = "icu")]
pub use fluent_static_formatter::format;

pub type FormatterFn =
    fn(&str, &fluent_static_value::Value, &mut dyn std::fmt::Write) -> std::fmt::Result;

#[cfg(not(feature = "icu"))]
pub fn format(
    _locale: &str,
    value: &fluent_static_value::Value,
    out: &mut impl std::fmt::Write,
) -> std::fmt::Result {
    match value {
        Value::String(s) => out.write_str(s),
        Value::Number { value, .. } => match value {
            Number::I64(n) => write!(out, "{}", n),
            Number::U64(n) => write!(out, "{}", n),
            Number::I128(n) => write!(out, "{}", n),
            Number::U128(n) => write!(out, "{}", n),
            Number::F64(n) => write!(out, "{}", n),
        },
        Value::Empty => Ok(()),
        Value::Error => write!(out, "#error#"),
    }
}
