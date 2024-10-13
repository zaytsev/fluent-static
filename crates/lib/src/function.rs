use std::{any::type_name, borrow::Cow, collections::HashMap};

use crate::Value;

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Function {fn_name} is called with unexpected number of positional params: expected {expected}, actual {actual}")]
    NumberOfPositionalParams {
        fn_name: &'static str,
        expected: usize,
        actual: usize,
    },

    #[error("Function {fn_name} doesn't support '{feature}' yet")]
    Unimplemented {
        fn_name: &'static str,
        feature: Cow<'static, str>,
    },

    #[error("Function {fn_name} called with unexpected argument {arg_name}")]
    InvalidArgument {
        fn_name: &'static str,
        arg_name: &'static str,
    },

    #[error("Function {fn_name} called with unexpected argument {arg_name} value {arg_value}")]
    InvalidArgumentValue {
        fn_name: &'static str,
        arg_name: &'static str,
        arg_value: Cow<'static, str>,
    },
}

pub trait FluentFn {
    const NAME: &'static str;

    fn type_name(&self) -> &'static str {
        type_name::<Self>()
    }

    fn apply<'a, 'b>(
        language_id: &'static str,
        positional_args: &'a [Value<'a>],
        named_args: &'a [(&'a str, Value<'a>)],
    ) -> Value<'b>;

    fn validate<'a>(
        positional_args: &'a [Value<'a>],
        named_args: &'a [(&'a str, Value<'a>)],
    ) -> Result<(), ValidationError>;
}

pub trait FluentFunction {
    fn type_name(&self) -> &'static str {
        type_name::<Self>()
    }
}

impl<'b, F> FluentFunction for F
where
    F: for<'a> Fn(&'static str, &'a [Value<'a>], &'a [(&'a str, Value<'a>)]) -> Value<'b> + 'static,
{
    // The default implementation from the trait will be used
}

pub trait FluentFunctionResolver {
    fn find(fluent_fn_name: &str) -> Option<&str>;
}

pub struct FluentFunctionRegistry {
    fns: HashMap<String, Vec<String>>,
}

impl FluentFunctionRegistry {
    pub fn register(&mut self, name: &str, f: impl FluentFunction) {
        self.fns
            .entry(name.to_string())
            .or_insert_with(|| Vec::new())
            .push(f.type_name().to_string());
    }
}

pub trait FunctionResolver {
    fn find(fn_name: &str) -> Option<impl FluentFn>;
}

pub mod builtins {
    use crate::function::ValidationError;

    use super::FluentFn;

    pub struct Number;

    impl FluentFn for Number {
        const NAME: &'static str = "NUMBER";

        fn apply<'a, 'b>(
            _language_id: &'static str,
            positional_args: &'a [crate::Value<'a>],
            _named_args: &'a [(&'a str, crate::Value<'a>)],
        ) -> crate::Value<'b> {
            assert!(
                positional_args.len() == 1,
                "NUMBER function expects exactly one positional argument"
            );
            todo!()
        }

        fn validate<'a>(
            positional_args: &'a [crate::Value<'a>],
            named_args: &'a [(&'a str, crate::Value<'a>)],
        ) -> Result<(), super::ValidationError> {
            if positional_args.len() != 1 {
                return Err(ValidationError::NumberOfPositionalParams {
                    fn_name: Self::NAME,
                    expected: 1,
                    actual: positional_args.len(),
                });
            }

            for (name, value) in named_args {
                match *name {
                    "style"
                    | "currency"
                    | "currencyDisplay"
                    | "useGrouping"
                    | "minimumIntegerDigits"
                    | "minimumFractionDigits"
                    | "maximumFractionDigits"
                    | "minimumSignificantDigits"
                    | "maximumSignificantDigits" => {
                        return Err(ValidationError::Unimplemented {
                            fn_name: Self::NAME,
                            feature: name.to_string().into(),
                        })
                    }
                    _ => {}
                }
            }

            Ok(())
        }
    }
}
