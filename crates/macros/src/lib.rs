use std::{collections::HashMap, env, ffi::OsString};

use fluent_static_codegen::{
    function::{FunctionCallGenerator, FunctionRegistry},
    MessageBundleBuilder,
};
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::{
    parse::Parse, parse_macro_input, punctuated::Punctuated, spanned::Spanned, token::Comma, Ident,
    ItemStruct, LitStr, Result as SyntaxResult, Token,
};

macro_rules! syntax_err {
    ($input:expr, $message:expr $(, $args:expr)*) => {
        ::syn::Error::new($input, format!($message $(, $args)*))
    }
}

#[proc_macro_attribute]
pub fn message_bundle(args: TokenStream, input: TokenStream) -> TokenStream {
    let item_struct = parse_macro_input!(input as ItemStruct);
    let name = item_struct.ident.to_string();
    let attrs = parse_macro_input!(args as MessageBundle);
    match attrs.builder.set_name(&name).build() {
        Ok(result) => {
            let includes: Vec<TokenStream2> = attrs
                .includes
                .iter()
                .map(|path| {
                    quote! {
                        #[cfg(trybuild)]
                        const _: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR_OVERRIDE"), "/", #path));
                        #[cfg(not(trybuild))]
                        const _: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", #path));
                    }
                })
                .collect();
            TokenStream::from(quote! {
                #(#includes)*
                #result
            })
        }
        Err(e) => syntax_err!(item_struct.span(), "Error generating message bundle: {}", e)
            .to_compile_error()
            .into(),
    }
}

fn get_project_dir() -> Option<OsString> {
    env::var_os("CARGO_MANIFEST_DIR_OVERRIDE") // used for tests
        .or_else(|| env::var_os("CARGO_MANIFEST_DIR"))
}

struct MessageBundle {
    builder: MessageBundleBuilder,
    includes: Vec<String>,
}

struct FluentResource {
    path: String,
    language: String,
    span: Span,
}

impl Parse for FluentResource {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let span = input.span();
        let content;
        syn::parenthesized!(content in input);
        let path: String = content.parse::<LitStr>()?.value();
        content.parse::<Token![,]>()?;
        let language: String = content.parse::<LitStr>()?.value();
        Ok(FluentResource {
            path,
            language,
            span,
        })
    }
}

struct FunctionMapping {
    fluent_id: LitStr,
    fn_ident: Option<Ident>,
}

impl Parse for FunctionMapping {
    fn parse(input: syn::parse::ParseStream) -> SyntaxResult<Self> {
        let fluent_id = input.parse::<LitStr>()?;
        let fn_ident = if input.peek(Token![=]) {
            input.parse::<Token![=]>()?;
            Some(input.parse::<Ident>()?)
        } else {
            None
        };
        Ok(FunctionMapping {
            fluent_id,
            fn_ident,
        })
    }
}

impl Parse for MessageBundle {
    fn parse(input: syn::parse::ParseStream) -> SyntaxResult<Self> {
        let base_dir = get_project_dir()
            .ok_or_else(|| syntax_err!(input.span(), "Unable to get project directory"))?;

        let mut fluent_resources: Vec<FluentResource> = Vec::new();
        let mut function_mappings: Vec<FunctionMapping> = Vec::new();
        let mut lang_def: Option<LitStr> = None;

        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            match ident.to_string().as_str() {
                "resources" => {
                    let resource_list;
                    syn::bracketed!(resource_list in input);
                    let resources: Punctuated<FluentResource, Comma> =
                        resource_list.parse_terminated(FluentResource::parse)?;
                    fluent_resources.extend(resources);
                }
                "default_language" => {
                    lang_def = Some(input.parse()?);
                }
                "functions" => {
                    let content;
                    syn::parenthesized!(content in input);
                    let fn_mappings: Punctuated<FunctionMapping, Comma> =
                        content.parse_terminated(FunctionMapping::parse)?;
                    function_mappings.extend(fn_mappings);
                }
                attr => return Err(syntax_err!(ident.span(), "Unexpected attribute {attr}")),
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        if fluent_resources.is_empty() {
            Err(syntax_err!(
                input.span(),
                "No Fluent resources defined. Missing or empty 'resources' attribute"
            ))
        } else if lang_def.is_none() {
            Err(syntax_err!(
                input.span(),
                "No default/fallback language is set. Missing 'default_language' attribute"
            ))
        } else {
            let mut builder = MessageBundleBuilder::default().with_base_dir(base_dir);
            let mut includes = Vec::new();

            builder = builder
                .with_default_language(&lang_def.unwrap().value())
                .map_err(|e| syntax_err!(input.span(), "Error parsing default language: {}", e))?;

            if !function_mappings.is_empty() {
                builder = builder.with_function_call_generator(BundleFunctionCallGenerator::new(
                    function_mappings,
                ));
            }

            for resource in fluent_resources {
                builder = builder
                    .add_resource(&resource.language, &resource.path)
                    .map_err(|e| syntax_err!(resource.span, "Error processing resource: {}", e))?;
                includes.push(resource.path);
            }

            Ok(MessageBundle { builder, includes })
        }
    }
}

struct BundleFunctionCallGenerator {
    fns: HashMap<String, TokenStream2>,
    registry: FunctionRegistry,
}

impl BundleFunctionCallGenerator {
    pub fn new(fn_mappings: Vec<FunctionMapping>) -> Self {
        let fns = fn_mappings
            .into_iter()
            .map(|mapping| {
                let ident = mapping
                    .fn_ident
                    .unwrap_or_else(|| format_ident!("{}", mapping.fluent_id.value()));
                (
                    mapping.fluent_id.value(),
                    quote! {
                        #ident
                    },
                )
            })
            .collect();

        let registry = FunctionRegistry::default();

        Self { fns, registry }
    }
}

impl FunctionCallGenerator for BundleFunctionCallGenerator {
    fn generate(
        &self,
        function_name: &str,
        positional_args: &Ident,
        named_args: &Ident,
    ) -> Option<TokenStream2> {
        if let Some(fn_ident) = self.fns.get(function_name) {
            Some(quote! {
                Self::#fn_ident(&#positional_args, &#named_args)
            })
        } else {
            self.registry
                .generate(function_name, positional_args, named_args)
        }
    }
}
