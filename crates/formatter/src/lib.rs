use std::fmt::Write;

use fluent_static_value::Value;

pub fn format(locale: &str, value: &Value, out: &mut impl Write) -> std::fmt::Result {
    match value {
        Value::String(s) => out.write_str(s),
        Value::Number { value, format } => number::format_number(locale, value, format, out),
        Value::Empty => Ok(()),
        Value::Error => write!(out, "#error#"),
    }
}

mod number {

    use std::{cell::RefCell, collections::HashMap, fmt::Write};

    use fluent_static_value::{
        number::format::{
            CurrencyDisplayStyle, CurrencySignMode, GroupingStyle, NumberStyle, UnitDisplayStyle,
        },
        Number, NumberFormat,
    };
    use rust_icu_unumberformatter::{UFormattedNumber, UNumberFormatter};

    thread_local! {
        static FORMATTER_CACHE: RefCell<HashMap<(String, NumberFormat), Result<UNumberFormatter, std::fmt::Error>>> = RefCell::new(HashMap::new());
    }

    pub(super) fn format_number(
        locale: &str,
        value: &Number,
        format: &Option<NumberFormat>,
        out: &mut impl Write,
    ) -> std::fmt::Result {
        if let Some(format) = format {
            let key = (locale.to_string(), format.clone());
            FORMATTER_CACHE.with(|cache| {
                let mut cache = cache.borrow_mut();
                if let Ok(formatter) = cache.entry(key).or_insert_with(|| {
                    let skeleton = make_icu_skeleton(format)?;
                    UNumberFormatter::try_new(&skeleton, locale).map_err(|_| std::fmt::Error)
                }) {
                    let formatted_number: UFormattedNumber = match value {
                        Number::I64(n) => formatter.format_int(*n),
                        n => formatter.format_double(n.as_f64()),
                    }
                    .map_err(|_| std::fmt::Error)?;
                    let s: String = formatted_number.try_into().map_err(|_| std::fmt::Error)?;
                    out.write_str(&s)?;
                    Ok(())
                } else {
                    Err(std::fmt::Error)
                }
            })
        } else {
            match value {
                Number::I64(n) => write!(out, "{}", n),
                Number::U64(n) => write!(out, "{}", n),
                Number::I128(n) => write!(out, "{}", n),
                Number::U128(n) => write!(out, "{}", n),
                Number::F64(n) => write!(out, "{}", n),
            }
        }
    }

    fn make_icu_skeleton(format: &NumberFormat) -> Result<String, std::fmt::Error> {
        let mut out = String::new();

        match &format.use_grouping {
            GroupingStyle::Always => {
                // TODO not supported by icu?
                // write!(out, "")?;
            }
            GroupingStyle::Auto => {
                write!(out, "group-auto ")?;
            }
            GroupingStyle::Min2 => {
                write!(out, "group-min2 ")?;
            }
            GroupingStyle::Off => {
                write!(out, "group-off ")?;
            }
        }

        match &format.style {
            NumberStyle::Decimal => {}
            NumberStyle::Currency { code, style, sign } => {
                write!(out, "currency/{} ", code)?;
                match style {
                    CurrencyDisplayStyle::Code => {
                        write!(out, "unit-width-iso-code ")?;
                    }
                    CurrencyDisplayStyle::Symbol => {
                        write!(out, "unit-width-short")?;
                    }
                    CurrencyDisplayStyle::NarrowSymbol => {
                        write!(out, "unit-width-narrow")?;
                    }
                    CurrencyDisplayStyle::Name => {
                        write!(out, "unit-width-full-name ")?;
                    }
                }
                match sign {
                    CurrencySignMode::Standard => {}
                    CurrencySignMode::Accounting => {
                        write!(out, "sign-accounting ")?;
                    }
                }
            }
            NumberStyle::Percent => write!(out, "percent")?,
            NumberStyle::Unit { identifier, style } => {
                write!(out, "unit/{} ", identifier)?;
                match style {
                    UnitDisplayStyle::Short => {
                        write!(out, "unit-width-short ")?;
                    }
                    UnitDisplayStyle::Narrow => {
                        write!(out, "unit-width-narrow ")?;
                    }
                    UnitDisplayStyle::Long => {
                        write!(out, "unit-width-full-name ")?;
                    }
                }
            }
        }

        if let Some(min_int_digits) = format.minimum_integer_digits.as_ref() {
            if *min_int_digits > 0 {
                write!(
                    out,
                    "integer-width/*{:0>width$} ",
                    "",
                    width = *min_int_digits
                )?;
            }
        }

        let min_frac = format.minimum_fraction_digits.clone();
        let max_frac = format.maximum_fraction_digits.clone();

        if let (Some(min_frac), Some(max_frac)) = (min_frac, max_frac) {
            if min_frac == max_frac {
                write!(out, ".{}", "0".repeat(min_frac))?;
            } else {
                write!(
                    out,
                    ".{}{}",
                    "0".repeat(min_frac),
                    "#".repeat(max_frac - min_frac)
                )?;
            }
        } else if let Some(min_frac) = min_frac {
            write!(out, ".{}*", "0".repeat(min_frac))?;
        } else if let Some(max_frac) = max_frac {
            write!(out, ".{}", "#".repeat(max_frac))?;
        }

        let min_sig = format.minimum_significant_digits.clone();
        let max_sig = format.maximum_significant_digits.clone();

        // Significant digits
        if let (Some(min_sig), Some(max_sig)) = (min_sig, max_sig) {
            if min_sig == max_sig {
                write!(out, "{}", "@".repeat(min_sig))?;
            } else {
                write!(
                    out,
                    "@{}{}",
                    "@".repeat(min_sig - 1),
                    "#".repeat(max_sig - min_sig)
                )?;
            }
        } else if let Some(min_sig) = min_sig {
            write!(out, "@{}*", "@".repeat(min_sig - 1))?;
        } else if let Some(max_sig) = max_sig {
            write!(out, "@{}", "#".repeat(max_sig - 1))?;
        }

        Ok(out)
    }

    #[cfg(test)]
    mod test {
        use fluent_static_value::{
            number::format::{
                CurrencyCode, CurrencyDisplayStyle, CurrencySignMode, GroupingStyle, NumberStyle,
                UnitDisplayStyle, UnitIdentifier,
            },
            Number, NumberFormat,
        };

        use crate::number::make_icu_skeleton;

        use super::format_number;

        #[test]
        fn default_format() {
            let mut s = String::new();
            format_number(
                "en-US",
                &Number::from(42),
                &Some(NumberFormat::default()),
                &mut s,
            )
            .expect("Number to be formatted");

            assert_eq!("42", s);
        }

        #[test]
        fn test_currency_format() {
            let mut s = String::new();
            format_number(
                "en-US",
                &Number::from(20),
                &Some(NumberFormat {
                    style: NumberStyle::Currency {
                        code: CurrencyCode::USD,
                        style: CurrencyDisplayStyle::Name,
                        sign: CurrencySignMode::default(),
                    },
                    ..Default::default()
                }),
                &mut s,
            )
            .expect("Number to be formatted");

            assert_eq!("20.00 US dollars", s);
        }

        #[test]
        fn test_currency_symbol() {
            let mut s = String::new();
            format_number(
                "en-US",
                &Number::from(20),
                &Some(NumberFormat {
                    style: NumberStyle::Currency {
                        code: CurrencyCode::USD,
                        style: CurrencyDisplayStyle::Symbol,
                        sign: CurrencySignMode::default(),
                    },
                    ..Default::default()
                }),
                &mut s,
            )
            .expect("Number to be formatted");

            assert_eq!("$20.00", s);
        }

        #[test]
        fn test_currency_narrow_symbol() {
            let mut s = String::new();
            format_number(
                "en-US",
                &Number::from(20),
                &Some(NumberFormat {
                    style: NumberStyle::Currency {
                        code: CurrencyCode::USD,
                        style: CurrencyDisplayStyle::NarrowSymbol,
                        sign: CurrencySignMode::default(),
                    },
                    ..Default::default()
                }),
                &mut s,
            )
            .expect("Number to be formatted");

            assert_eq!("$20.00", s);
        }

        #[test]
        fn test_unit_short() {
            let mut s = String::new();
            format_number(
                "en-US",
                &Number::from(20),
                &Some(NumberFormat {
                    style: NumberStyle::Unit {
                        identifier: UnitIdentifier::Meter,
                        style: UnitDisplayStyle::Short,
                    },
                    ..Default::default()
                }),
                &mut s,
            )
            .expect("Number to be formatted");

            assert_eq!("20 m", s);
        }

        #[test]
        fn test_unit_long() {
            let mut s = String::new();
            format_number(
                "en-US",
                &Number::from(20),
                &Some(NumberFormat {
                    style: NumberStyle::Unit {
                        identifier: UnitIdentifier::Meter,
                        style: UnitDisplayStyle::Long,
                    },
                    ..Default::default()
                }),
                &mut s,
            )
            .expect("Number to be formatted");

            assert_eq!("20 meters", s);
        }

        #[test]
        fn test_unit_narrow() {
            let mut s = String::new();
            format_number(
                "en-US",
                &Number::from(20),
                &Some(NumberFormat {
                    style: NumberStyle::Unit {
                        identifier: UnitIdentifier::Meter,
                        style: UnitDisplayStyle::Narrow,
                    },
                    ..Default::default()
                }),
                &mut s,
            )
            .expect("Number to be formatted");

            assert_eq!("20m", s);
        }

        #[test]
        fn test_unit_compound_short() {
            let mut s = String::new();
            format_number(
                "en-US",
                &Number::from(20),
                &Some(NumberFormat {
                    style: NumberStyle::Unit {
                        identifier: UnitIdentifier::Mile.per(UnitIdentifier::Hour),
                        style: UnitDisplayStyle::Short,
                    },
                    ..Default::default()
                }),
                &mut s,
            )
            .expect("Number to be formatted");

            assert_eq!("20 mph", s);
        }

        #[test]
        fn test_min_integer() {
            let mut s = String::new();
            format_number(
                "en-US",
                &Number::from(20),
                &Some(NumberFormat {
                    minimum_integer_digits: Some(5),
                    use_grouping: GroupingStyle::Off,
                    ..Default::default()
                }),
                &mut s,
            )
            .expect("Number to be formatted");

            assert_eq!("00020", s);
        }

        #[test]
        fn test_fraction_digits() {
            let test_data: Vec<(f64, Option<usize>, Option<usize>, &str)> = vec![
                (0.12345, Some(2), None, "0.12345"),
                (0.12345, None, Some(2), "0.12"),
                (0.1, Some(3), None, "0.100"),
                (0.12345, Some(3), Some(4), "0.1234"),
            ];

            for (n, min, max, expected) in test_data {
                let mut s = String::new();
                format_number(
                    "en-US",
                    &Number::from(n),
                    &Some(NumberFormat {
                        use_grouping: GroupingStyle::Off,
                        minimum_fraction_digits: min.clone(),
                        maximum_fraction_digits: max.clone(),
                        ..Default::default()
                    }),
                    &mut s,
                )
                .expect("Number to be formatted");

                assert_eq!(
                    expected,
                    s,
                    "n={}, min={}, max={}",
                    n,
                    min.map(|min| min.to_string()).unwrap_or_default(),
                    max.map(|max| max.to_string()).unwrap_or_default()
                );
            }
        }

        #[test]
        fn test_significant_digits() {
            let test_data: Vec<(f64, Option<usize>, Option<usize>, &str)> = vec![
                (123.45, None, None, "123.45"),
                (0.12345, None, None, "0.12345"),
                (123.45, Some(2), None, "123.45"),
                (12.345, Some(3), None, "12.345"),
                (1.2345, Some(4), None, "1.2345"),
                (0.12345, Some(5), None, "0.12345"),
                (0.001234, Some(3), None, "0.001234"),
                (123.45, None, Some(2), "120"),
                (12.345, None, Some(3), "12.3"),
                // (1.2345, None, Some(4), "1.235"),
                (0.12345, None, Some(5), "0.12345"),
                (0.001234, None, Some(3), "0.00123"),
                // (123.45, Some(2), Some(4), "123.5"),
                (12.345, Some(3), Some(5), "12.345"),
                (1.2345, Some(4), Some(6), "1.2345"),
                // (0.12345, Some(3), Some(4), "0.1235"),
                (0.001234, Some(2), Some(3), "0.00123"),
                (0.0, Some(2), None, "0.0"),
                (0.0, None, Some(2), "0"),
                (0.0, Some(2), Some(3), "0.0"),
                (123.45, Some(3), Some(3), "123"),
                (0.12345, Some(3), Some(3), "0.123"),
            ];

            let mut i = 0;

            for (n, min, max, expected) in test_data {
                let mut s = String::new();
                let format = NumberFormat {
                    use_grouping: GroupingStyle::Off,
                    minimum_significant_digits: min.clone(),
                    maximum_significant_digits: max.clone(),
                    ..Default::default()
                };

                format_number("en-US", &Number::from(n), &Some(format.clone()), &mut s)
                    .expect("Number to be formatted");

                assert_eq!(
                    expected,
                    s,
                    "{}: n={}, min={}, max={}, skel={}",
                    i,
                    n,
                    min.map(|min| min.to_string()).unwrap_or_default(),
                    max.map(|max| max.to_string()).unwrap_or_default(),
                    make_icu_skeleton(&format).unwrap()
                );

                i = i + 1;
            }
        }
    }
}
