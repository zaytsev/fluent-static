use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NumberStyle {
    Decimal,
    Currency {
        code: CurrencyCode,
        style: Option<CurrencyDisplayStyle>,
        sign: Option<CurrencySignMode>,
    },
    Percent,
    Unit {
        identifier: UnitIdentifier,
        style: Option<UnitDisplayStyle>,
    },
}

impl Default for NumberStyle {
    fn default() -> Self {
        Self::Decimal
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Invalid currency code: '{0}")]
pub struct InvalidCurrencyCode(String);

macro_rules! create_currency_code_enum {
    ($($code:ident),*) => {
        #[derive(Debug, PartialEq, Eq, Clone, Copy)]
        pub enum CurrencyCode {
            $($code),*
        }

        impl ::std::str::FromStr for CurrencyCode {
            type Err = $crate::value::format::number::InvalidCurrencyCode;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $(stringify!($code) => Ok(CurrencyCode::$code),)*
                    _ => Err(InvalidCurrencyCode(s.to_string())),
                }
            }
        }

        impl ::std::fmt::Display for CurrencyCode {
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

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
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

macro_rules! generate_unit_identifier {
    ($($raw_id:ident),*) => {
        ::paste::paste! {
            #[derive(Debug, Clone, Copy, PartialEq, Eq)]
            pub enum UnitIdentifier {
                $( [< $raw_id:camel >] ),*
            }

            impl ::std::str::FromStr for UnitIdentifier {
                type Err = ();

                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    match s.replace('-', "_").to_lowercase().as_str() {
                        $(stringify!($raw_id) => Ok(UnitIdentifier::[< $raw_id:camel >]),)*
                        _ => Err(()),
                    }
                }
            }

            impl ::std::fmt::Display for UnitIdentifier {
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    let s = match self {
                        $(UnitIdentifier::[< $raw_id:camel >] => stringify!($raw_id)),*
                    }.replace('_', "-");
                    f.write_str(s.as_str())
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
    fluid_ounce,
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
    mile_scandinavian,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupingStyle {
    Always,
    Auto,
    Min2,
}

impl Default for GroupingStyle {
    fn default() -> Self {
        Self::Auto
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Invalid grouping style: '{0}")]
pub struct InvalidGroupingStyleError(String);

impl FromStr for GroupingStyle {
    type Err = InvalidGroupingStyleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "auto" | "true" | "false" => Ok(Self::Auto),
            "always" => Ok(Self::Always),
            "min2" => Ok(Self::Min2),
            _ => Err(InvalidGroupingStyleError(s.to_string())),
        }
    }
}

#[derive(Debug, Clone)]
pub struct NumberFormat {
    pub style: Option<NumberStyle>,

    pub use_grouping: Option<GroupingStyle>,

    pub minimum_integer_digits: Option<usize>,
    pub minimum_fraction_digits: Option<usize>,
    pub maximum_fraction_digits: Option<usize>,
    pub minimum_significant_digits: Option<usize>,
    pub maximum_significant_digits: Option<usize>,
}
