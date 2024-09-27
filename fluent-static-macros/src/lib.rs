use std::{env, ffi::OsString};

use fluent_static_bundle::MessageBundleBuilder;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
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
}

impl Parse for FluentResource {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        syn::parenthesized!(content in input);
        let path: String = content.parse::<LitStr>()?.value();
        content.parse::<Token![,]>()?;
        let language: String = content.parse::<LitStr>()?.value();
        Ok(FluentResource { path, language })
    }
}

impl Parse for MessageBundle {
    fn parse(input: syn::parse::ParseStream) -> SyntaxResult<Self> {
        let base_dir = get_project_dir()
            .ok_or_else(|| syntax_err!(input.span(), "Unable to get project directory"))?;
        let mut builder = MessageBundleBuilder::default().with_base_dir(base_dir);

        let mut includes = Vec::new();

        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            match ident.to_string().as_str() {
                "resources" => {
                    let resource_list;
                    syn::bracketed!(resource_list in input);
                    let resources: Punctuated<FluentResource, Comma> =
                        resource_list.parse_terminated(FluentResource::parse)?;

                    for resource in resources {
                        builder = builder
                            .add_resource(&resource.language, &resource.path)
                            .map_err(|e| {
                                syntax_err!(
                                    resource_list.span(),
                                    "Error processing resource: {}",
                                    e
                                )
                            })?;
                        includes.push(resource.path);
                    }
                }
                "default_language" => {
                    let lang: LitStr = input.parse()?;
                    builder = builder.with_default_language(&lang.value()).map_err(|e| {
                        syntax_err!(input.span(), "Error parsing default language: {}", e)
                    })?;
                }
                attr => return Err(syntax_err!(ident.span(), "Unexpected attribute {attr}")),
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(MessageBundle { builder, includes })
    }
}
