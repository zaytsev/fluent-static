use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NumberStyle {
    Decimal,
    Currency {
        code: CurrencyCode,
        style: CurrencyDisplayStyle,
        sign: CurrencySignMode,
    },
    Percent,
    Unit {
        identifier: UnitIdentifier,
        style: UnitDisplayStyle,
    },
}

impl Default for NumberStyle {
    fn default() -> Self {
        Self::Decimal
    }
}

impl NumberStyle {
    pub fn is_currency(&self) -> bool {
        match self {
            NumberStyle::Currency { .. } => true,
            _ => false,
        }
    }

    pub fn is_unit(&self) -> bool {
        match self {
            NumberStyle::Unit { .. } => true,
            _ => false,
        }
    }

    pub fn is_decimal(&self) -> bool {
        match self {
            NumberStyle::Decimal => true,
            _ => false,
        }
    }

    pub fn is_percent(&self) -> bool {
        match self {
            NumberStyle::Percent => true,
            _ => false,
        }
    }

    pub fn set_currency_display_style(&mut self, new_style: CurrencyDisplayStyle) -> bool {
        if let NumberStyle::Currency { style, .. } = self {
            *style = new_style;
            true
        } else {
            false
        }
    }

    pub fn set_currency_sign_mode(&mut self, new_sign: CurrencySignMode) -> bool {
        if let NumberStyle::Currency { sign, .. } = self {
            *sign = new_sign;
            true
        } else {
            false
        }
    }

    pub fn set_unit_display_style(&mut self, new_style: UnitDisplayStyle) -> bool {
        if let NumberStyle::Unit { style, .. } = self {
            *style = new_style;
            true
        } else {
            false
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Invalid currency code: '{0}")]
pub struct InvalidCurrencyCode(String);

macro_rules! create_currency_code_enum {
    ($($code:ident),*) => {
        #[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
        pub enum CurrencyCode {
            $($code),*
        }

        impl FromStr for CurrencyCode {
            type Err = InvalidCurrencyCode;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $(stringify!($code) => Ok(CurrencyCode::$code),)*
                    _ => Err(InvalidCurrencyCode(s.to_string())),
                }
            }
        }

        impl Display for CurrencyCode {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                let s = match self {
                    $(CurrencyCode::$code => stringify!($code)),*
                };
                f.write_str(s)
            }
        }
    };
}

create_currency_code_enum!(
    AED, AFN, ALL, AMD, ANG, AOA, ARS, AUD, AWG, AZN, BAM, BBD, BDT, BGN, BHD, BIF, BMD, BND, BOB,
    BOV, BRL, BSD, BTN, BWP, BYN, BZD, CAD, CDF, CHE, CHF, CHW, CLF, CLP, CNY, COP, COU, CRC, CUP,
    CVE, CZK, DJF, DKK, DOP, DZD, EGP, ERN, ETB, EUR, FJD, FKP, GBP, GEL, GHS, GIP, GMD, GNF, GTQ,
    GYD, HKD, HNL, HTG, HUF, IDR, ILS, INR, IQD, IRR, ISK, JMD, JOD, JPY, KES, KGS, KHR, KMF, KPW,
    KRW, KWD, KYD, KZT, LAK, LBP, LKR, LRD, LSL, LYD, MAD, MDL, MGA, MKD, MMK, MNT, MOP, MRU, MUR,
    MVR, MWK, MXN, MXV, MYR, MZN, NAD, NGN, NIO, NOK, NPR, NZD, OMR, PAB, PEN, PGK, PHP, PKR, PLN,
    PYG, QAR, RON, RSD, RUB, RWF, SAR, SBD, SCR, SDG, SEK, SGD, SHP, SLE, SOS, SRD, SSP, STN, SVC,
    SYP, SZL, THB, TJS, TMT, TND, TOP, TRY, TTD, TWD, TZS, UAH, UGX, USD, USN, UYI, UYU, UYW, UZS,
    VED, VES, VND, VUV, WST, XAF, XAG, XAU, XBA, XBB, XBC, XBD, XCD, XDR, XOF, XPD, XPF, XPT, XSU,
    XTS, XUA, XXX, YER, ZAR, ZMW, ZWG
);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CurrencyDisplayStyle {
    Code,
    Symbol,
    NarrowSymbol,
    Name,
}

#[derive(Debug, thiserror::Error)]
#[error("Invalid currency display style: '{0}'")]
pub struct InvalidCurrencyDisplayStyleError(String);

impl FromStr for CurrencyDisplayStyle {
    type Err = InvalidCurrencyDisplayStyleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "code" => Ok(CurrencyDisplayStyle::Code),
            "symbol" => Ok(CurrencyDisplayStyle::Symbol),
            "narrowsymbol" => Ok(CurrencyDisplayStyle::NarrowSymbol),
            "name" => Ok(CurrencyDisplayStyle::Name),
            _ => Err(InvalidCurrencyDisplayStyleError(s.to_string())),
        }
    }
}

impl Default for CurrencyDisplayStyle {
    fn default() -> Self {
        Self::Symbol
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CurrencySignMode {
    Standard,
    Accounting,
}

#[derive(Debug, thiserror::Error)]
#[error("Invalid currency sign mode: '{0}'")]
pub struct InvalidCurrencySignModeError(String);

impl FromStr for CurrencySignMode {
    type Err = InvalidCurrencySignModeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "standard" => Ok(CurrencySignMode::Standard),
            "accounting" => Ok(CurrencySignMode::Accounting),
            _ => Err(InvalidCurrencySignModeError(s.to_string())),
        }
    }
}

impl Default for CurrencySignMode {
    fn default() -> Self {
        Self::Standard
    }
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("Invalid currency sign mode: '{0}'")]
pub struct InvalidUnitIdentifierError(String);

macro_rules! generate_unit_identifier {
    ($($raw_id:ident),*) => {
        ::paste::paste! {
            #[derive(Debug, Clone, PartialEq, Eq, Hash)]
            pub enum UnitIdentifier {
                Derived(Box<UnitIdentifier>, Box<UnitIdentifier>),
                $( [< $raw_id:camel >] ),*
            }

            impl UnitIdentifier {
                pub fn per(&self, denominator: UnitIdentifier) -> UnitIdentifier {
                    Self::Derived(Box::new(self.clone()), Box::new(denominator))
                }
            }

            impl FromStr for UnitIdentifier {
                type Err = InvalidUnitIdentifierError;

                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    match s {
                        $(stringify!($raw_id) => Ok(UnitIdentifier::[< $raw_id:camel >]),)*
                        _ => if let Some(index) = s.find("-per-") {
                                let (left_str, right_str) = s.split_at(index);
                                let right_str = &right_str[5..]; // Skip the "-per-" part

                                let left = UnitIdentifier::from_str(left_str)?;
                                let right = UnitIdentifier::from_str(right_str)?;

                                Ok(UnitIdentifier::Derived(Box::new(left), Box::new(right)))
                            } else {
                                Err(InvalidUnitIdentifierError(s.to_string()))
                        }
                    }
                }
            }

            impl Display for UnitIdentifier {
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    match self {
                        UnitIdentifier::Derived(n, d) => write!(f, "{}-per-{}", n, d),
                        $(UnitIdentifier::[< $raw_id:camel >] => f.write_str(stringify!($raw_id))),*
                    }
                }
            }
        }
    };
}

// Generate the UnitIdentifier enum and its implementations
generate_unit_identifier!(
    acre,
    bit,
    byte,
    celsius,
    centimeter,
    day,
    degree,
    fahrenheit,
    foot,
    gallon,
    gigabit,
    gigabyte,
    gram,
    hectare,
    hour,
    inch,
    kilobit,
    kilobyte,
    kilogram,
    kilometer,
    liter,
    megabit,
    megabyte,
    meter,
    microsecond,
    mile,
    milliliter,
    millimeter,
    millisecond,
    minute,
    month,
    nanosecond,
    ounce,
    percent,
    petabyte,
    pound,
    second,
    stone,
    terabit,
    terabyte,
    week,
    yard,
    year
);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnitDisplayStyle {
    Short,
    Narrow,
    Long,
}

impl Default for UnitDisplayStyle {
    fn default() -> Self {
        Self::Short
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GroupingStyle {
    Always,
    Auto,
    Min2,
    Off,
}

impl Default for GroupingStyle {
    fn default() -> Self {
        Self::Auto
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Invalid unit display style: '{0}")]
pub struct InvalidUnitDisplayStyleError(String);

impl FromStr for UnitDisplayStyle {
    type Err = InvalidUnitDisplayStyleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "short" => Ok(UnitDisplayStyle::Short),
            "narrow" => Ok(UnitDisplayStyle::Narrow),
            "long" => Ok(UnitDisplayStyle::Long),
            _ => Err(InvalidUnitDisplayStyleError(s.to_string())),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Invalid grouping style: '{0}")]
pub struct InvalidGroupingStyleError(String);

impl FromStr for GroupingStyle {
    type Err = InvalidGroupingStyleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "auto" | "true" => Ok(Self::Auto),
            "off" | "false" => Ok(Self::Off),
            "always" => Ok(Self::Always),
            "min2" => Ok(Self::Min2),
            _ => Err(InvalidGroupingStyleError(s.to_string())),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NumberFormat {
    pub style: NumberStyle,

    pub use_grouping: GroupingStyle,

    pub minimum_integer_digits: Option<usize>,
    pub minimum_fraction_digits: Option<usize>,
    pub maximum_fraction_digits: Option<usize>,
    pub minimum_significant_digits: Option<usize>,
    pub maximum_significant_digits: Option<usize>,
}

impl NumberFormat {
    pub fn currency(code: CurrencyCode) -> Self {
        Self {
            style: NumberStyle::Currency {
                code,
                style: CurrencyDisplayStyle::default(),
                sign: CurrencySignMode::default(),
            },
            ..Default::default()
        }
    }

    pub fn unit(unit_id: UnitIdentifier) -> Self {
        Self {
            style: NumberStyle::Unit {
                identifier: unit_id,
                style: UnitDisplayStyle::default(),
            },
            ..Default::default()
        }
    }

    pub fn percent() -> Self {
        Self {
            style: NumberStyle::Percent,
            ..Default::default()
        }
    }
}

impl Default for NumberFormat {
    fn default() -> Self {
        Self {
            style: NumberStyle::Decimal,
            use_grouping: GroupingStyle::Auto,
            minimum_integer_digits: None,
            minimum_fraction_digits: None,
            maximum_fraction_digits: None,
            minimum_significant_digits: None,
            maximum_significant_digits: None,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::number::format::UnitIdentifier;

    #[test]
    fn test_unit_from_str() {
        assert_eq!(UnitIdentifier::Meter, "meter".parse().unwrap());
        assert_eq!(
            UnitIdentifier::Meter.per(UnitIdentifier::Second),
            "meter-per-second".parse().unwrap()
        );
    }

    #[test]
    fn test_unit_display() {
        assert_eq!(
            "kilometer-per-hour",
            format!("{}", UnitIdentifier::Kilometer.per(UnitIdentifier::Hour))
        );
    }
}
