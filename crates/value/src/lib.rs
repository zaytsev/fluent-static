use std::{borrow::Cow, str::FromStr};

pub mod number;

pub use number::format::NumberFormat;
pub use number::Number;

#[derive(Debug, Clone)]
pub enum Value<'a> {
    String(Cow<'a, str>),
    Number {
        value: Number,
        format: Option<NumberFormat>,
    },
    // TODO datetime
    Empty,
    Error,
}

impl<'a> Value<'a> {
    pub fn try_number(value: &'a str) -> Self {
        if let Ok(number) = Number::from_str(value) {
            number.into()
        } else {
            value.into()
        }
    }

    pub fn formatted_number(value: impl Into<Number>, number_format: NumberFormat) -> Self {
        Self::Number {
            value: value.into(),
            format: Some(number_format),
        }
    }

    pub fn is_string(&self) -> bool {
        match self {
            Value::String(_) => true,
            _ => false,
        }
    }

    pub fn is_number(&self) -> bool {
        match self {
            Value::Number { .. } => true,
            _ => false,
        }
    }
}

impl<'a> PartialEq for Value<'a> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::String(s), Value::String(o)) => s == o,
            (
                Value::Number {
                    value: self_value, ..
                },
                Value::Number {
                    value: other_value, ..
                },
            ) => self_value == other_value,
            _ => false,
        }
    }
}

impl<'a> From<String> for Value<'a> {
    fn from(value: String) -> Self {
        Self::String(value.into())
    }
}

impl<'a> From<&'a String> for Value<'a> {
    fn from(value: &'a String) -> Self {
        Self::String(value.into())
    }
}

impl<'a> From<&'a str> for Value<'a> {
    fn from(value: &'a str) -> Self {
        Self::String(value.into())
    }
}

impl<'a> From<Cow<'a, str>> for Value<'a> {
    fn from(value: Cow<'a, str>) -> Self {
        Self::String(value)
    }
}

impl<'a, T> From<Option<T>> for Value<'a>
where
    T: Into<Value<'a>>,
{
    fn from(value: Option<T>) -> Self {
        match value {
            Some(v) => v.into(),
            None => Self::Empty,
        }
    }
}
