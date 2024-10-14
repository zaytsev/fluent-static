use std::collections::HashMap;

use fluent_static_function::FluentFunctionDescriptor;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

pub trait FunctionCallGenerator {
    fn generate(
        &self,
        function_name: &str,
        positional_args: &Ident,
        named_args: &Ident,
    ) -> Option<TokenStream>;
}

pub struct FunctionRegistry {
    fns: HashMap<String, TokenStream>,
}

impl FunctionRegistry {
    pub fn register(
        &mut self,
        function_id: &str,
        function_descriptor: impl FluentFunctionDescriptor,
    ) -> &Self {
        self.fns
            .insert(function_id.to_string(), Self::fqn(function_descriptor));
        self
    }

    fn fqn(function_descriptor: impl FluentFunctionDescriptor) -> TokenStream {
        let path = function_descriptor.type_name();

        // HACK fluent_static reexports fluent_static_function,
        // need to patch function name
        // to avoid fluent_static_function dependency
        let path = if path.starts_with("fluent_static_function::builtins") {
            path.replace("fluent_static_function", "fluent_static::function")
        } else {
            path.to_string()
        };

        let parts: Vec<&str> = path.split("::").collect();
        let idents: Vec<_> = parts.iter().map(|part| format_ident!("{}", part)).collect();

        quote! { ::#(#idents)::* }
    }
}

impl Default for FunctionRegistry {
    fn default() -> Self {
        let fns = HashMap::new();
        let mut result = Self { fns };

        result.register("NUMBER", fluent_static_function::builtins::number);

        result
    }
}

impl FunctionCallGenerator for FunctionRegistry {
    fn generate(
        &self,
        function_name: &str,
        positional_args: &Ident,
        named_args: &Ident,
    ) -> Option<TokenStream> {
        if let Some(fn_ident) = self.fns.get(function_name) {
            Some(quote! {
                #fn_ident(&#positional_args, &#named_args)
            })
        } else {
            None
        }
    }
}
