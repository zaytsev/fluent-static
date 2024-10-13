use std::{borrow::Cow, str::FromStr};

use format::number::NumberFormat;

pub mod format;

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

#[derive(Debug, Clone, Copy)]
pub enum Number {
    I64(i64),
    U64(u64),
    I128(i128),
    U128(u128),
    F64(f64),
}

impl PartialEq for Number {
    fn eq(&self, other: &Self) -> bool {
        match self.ordered_tuple(other) {
            (Number::I64(i1), Number::I64(i2)) => i1 == i2,
            (Number::U64(u1), Number::I64(i2)) if *i2 >= 0 => *u1 == *i2 as u64,
            (Number::U64(u1), Number::U64(u2)) => u1 == u2,
            (Number::I128(i1), Number::I64(i2)) => *i1 == *i2 as i128,
            (Number::I128(i1), Number::U64(u2)) => *i1 == *u2 as i128,
            (Number::I128(i1), Number::I128(i2)) => i1 == i2,
            (Number::U128(u1), Number::I64(i2)) if *i2 >= 0 => *u1 == *i2 as u128,
            (Number::U128(u1), Number::U64(u2)) => *u1 == *u2 as u128,
            (Number::U128(u1), Number::I128(i2)) if *i2 >= 0 => *u1 == *i2 as u128,
            (Number::U128(u1), Number::U128(u2)) => u1 == u2,
            (Number::F64(f1), n2) => f64::abs(f1 - n2.as_f64()) < f64::EPSILON,

            _ => false,
        }
    }
}

impl Number {
    fn ord(&self) -> usize {
        match self {
            Number::I64(_) => 0,
            Number::U64(_) => 1,
            Number::I128(_) => 2,
            Number::U128(_) => 3,
            Number::F64(_) => 4,
        }
    }

    fn ordered_tuple<'a>(&'a self, other: &'a Self) -> (&'a Self, &'a Self) {
        if self.ord() > other.ord() {
            (self, other)
        } else {
            (other, self)
        }
    }

    pub fn as_f64(&self) -> f64 {
        match self {
            Number::I64(v) => *v as f64,
            Number::U64(v) => *v as f64,
            Number::I128(v) => *v as f64,
            Number::U128(v) => *v as f64,
            Number::F64(v) => *v,
        }
    }

    pub fn as_string(&self) -> String {
        match self {
            Number::I64(n) => n.to_string(),
            Number::U64(n) => n.to_string(),
            Number::I128(n) => n.to_string(),
            Number::U128(n) => n.to_string(),
            Number::F64(n) => n.to_string(),
        }
    }
}

impl FromStr for Number {
    type Err = std::num::ParseFloatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains('.') {
            // If the string contains a decimal point, parse as f64
            match s.parse::<f64>() {
                Ok(f) => Ok(Number::F64(f)),
                Err(e) => Err(e),
            }
        } else {
            // Try parsing as integers in order: i64, u64, i128, u128
            if let Ok(i) = s.parse::<i64>() {
                Ok(Number::I64(i))
            } else if let Ok(u) = s.parse::<u64>() {
                Ok(Number::U64(u))
            } else if let Ok(i) = s.parse::<i128>() {
                Ok(Number::I128(i))
            } else if let Ok(u) = s.parse::<u128>() {
                Ok(Number::U128(u))
            } else {
                // If all integer parsing fails, try f64 as a fallback
                match s.parse::<f64>() {
                    Ok(f) => Ok(Number::F64(f)),
                    Err(e) => Err(e),
                }
            }
        }
    }
}

impl<'a> From<Number> for Value<'a> {
    fn from(value: Number) -> Self {
        Self::Number {
            value,
            format: None,
        }
    }
}

macro_rules! impl_from_for_number {
    ($($t:ty => $variant:ident),*) => {
        $(
            impl From<$t> for Number {
                fn from(value: $t) -> Self {
                    Number::$variant(value as _)
                }
            }
            impl From<&$t> for Number {
                fn from(value: &$t) -> Self {
                    Number::$variant(*value as _)
                }
            }
            impl From<$t> for Value<'_> {
                fn from(value: $t) -> Self {
                    Value::Number {
                        value: Number::$variant(value as _),
                        format: None
                    }
                }
            }
            impl From<&$t> for Value<'_> {
                fn from(value: &$t) -> Self {
                    Value::Number{
                        value: Number::$variant(*value as _),
                        format: None
                    }
                }
            }
        )*
    };
}

impl_from_for_number! {
    i8 => I64,
    i16 => I64,
    i32 => I64,
    i64 => I64,
    i128 => I128,
    isize => I64,
    u8 => U64,
    u16 => U64,
    u32 => U64,
    u64 => U64,
    u128 => U128,
    usize => U64,
    f32 => F64,
    f64 => F64
}

#[cfg(test)]
mod test {
    use crate::value::Number;

    #[test]
    fn test_number_equality() {
        let n1 = Number::I64(0);
        let n2 = Number::F64(0f64);
        assert_eq!(n1, n2);
    }
}
