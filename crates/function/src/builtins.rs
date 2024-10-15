use std::str::FromStr;

use fluent_static_value::{number::format::GroupingStyle, Number, NumberFormat, Value};

pub fn number<'a, 'b>(
    positional_args: &'a [Value<'a>],
    named_args: &'a [(&'a str, Value<'a>)],
) -> Value<'b> {
    if let Some(value) = positional_args.get(0) {
        match value {
            Value::String(s) => Number::from_str(s)
                .map(|n| Value::Number {
                    value: n,
                    format: Some(parse_number_format(None, named_args)),
                })
                .unwrap_or(Value::Error),
            Value::Number { value, format } => Value::Number {
                value: value.clone(),
                format: Some(parse_number_format(format.clone(), named_args)),
            },
            Value::Empty => Value::Empty,
            Value::Error => Value::Error,
        }
    } else {
        Value::Error
    }
}

fn parse_number_format<'a>(
    value_format: Option<NumberFormat>,
    named_args: &'a [(&'a str, Value<'a>)],
) -> NumberFormat {
    let mut result = value_format.unwrap_or_default();
    for (key, value) in named_args {
        match *key {
            "currencyDisplay" => {}
            "unitDisplay" => {}
            "useGrouping" => {
                if let Value::String(s) = value {
                    result.use_grouping = GroupingStyle::from_str(s).ok().or(result.use_grouping);
                };
            }
            "minimumIntegerDigits" => result.minimum_integer_digits = read_digits(value, 1, 21),
            "minimumFractionDigits" => result.minimum_fraction_digits = read_digits(value, 0, 100),
            "maximumFractionDigits" => result.maximum_fraction_digits = read_digits(value, 0, 100),
            "minimumSignificantDigits" => {
                result.minimum_significant_digits = read_digits(value, 1, 21)
            }
            "maximumSignificantDigits" => {
                result.maximum_significant_digits = read_digits(value, 1, 21)
            }
            _ => {}
        }
    }
    result
}

fn read_digits<'a>(value: &Value<'a>, min: usize, max: usize) -> Option<usize> {
    match value {
        Value::String(s) => Number::from_str(s).ok().map(|n| clamp(&n, min, max)),
        Value::Number { value, .. } => Some(clamp(value, min, max)),
        _ => None,
    }
}

fn clamp(value: &Number, min: usize, max: usize) -> usize {
    match value {
        Number::I64(val) => (*val).clamp(min as i64, max as i64) as usize,
        Number::U64(val) => (*val).clamp(min as u64, max as u64) as usize,
        Number::I128(val) => (*val).clamp(min as i128, max as i128) as usize,
        Number::U128(val) => (*val).clamp(min as u128, max as u128) as usize,
        Number::F64(val) => {
            let clamped = val.clamp(min as f64, max as f64);
            clamped.round() as usize
        }
    }
}
