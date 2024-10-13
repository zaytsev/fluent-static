use std::{borrow::Cow, collections::HashMap};

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

pub enum Arg {
    String,
    Number,
    Placeholder,
}

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

pub trait ArgumentsValidator {
    fn validate(function_name: &str, positional_args: Vec<Arg>, named_args: HashMap<&str, Arg>);
}

pub struct BuiltinFunctionsValidator {}

impl ArgumentsValidator for BuiltinFunctionsValidator {
    fn validate(function_name: &str, positional_args: Vec<Arg>, named_args: HashMap<&str, Arg>) {
        match function_name {
            "NUMBER" => todo!(),
            _ => todo!(),
        }
    }
}

pub trait CodeGenerator {
    fn generate_call(
        &self,
        function_name: &str,
        positional_args: Ident,
        named_args: Ident,
    ) -> Option<TokenStream>;
}

pub struct Registry {
    fns: HashMap<String, Ident>,
}

impl Registry {
    pub fn register(&mut self, fluent_function_name: &str, rust_function_name: &str) -> &Self {
        self.fns.insert(
            fluent_function_name.to_string(),
            format_ident!("{}", rust_function_name),
        );
        self
    }
}

impl Default for Registry {
    fn default() -> Self {
        let mut fns = HashMap::new();
        fns.insert(
            "NUMBER".to_string(),
            format_ident!("::fluent_static::function::builtins::number"),
        );

        Self { fns }
    }
}

impl CodeGenerator for Registry {
    fn generate_call(
        &self,
        function_name: &str,
        positional_args: Ident,
        named_args: Ident,
    ) -> Option<TokenStream> {
        if let Some(fn_ident) = self.fns.get(function_name) {
            Some(quote! {
                #fn_ident(#positional_args, #named_args)
            })
        } else {
            None
        }
    }
}
