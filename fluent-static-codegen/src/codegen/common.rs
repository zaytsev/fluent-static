use proc_macro2::TokenStream;
use syn::Ident;

use crate::{bundle::MessageBundle, Error};
use quote::quote;

pub fn language_bundle_definitions(bundle: &MessageBundle) -> Vec<TokenStream> {
    bundle.langs.iter().map(|language_bundle| {
            let lang_id = language_bundle.language();
            let resource_ident = language_bundle.static_resource_ident();
            let bundle_ident = language_bundle.static_bundle_ident();
            let resource = language_bundle.resource_literal();
            quote! {
                static #resource_ident: &str = #resource;
                static #bundle_ident: Lazy<FluentBundle<FluentResource>> = Lazy::new(|| {
                    let lang_id = fluent_static::unic_langid::langid!(#lang_id);
                    let mut bundle: FluentBundle<FluentResource> = FluentBundle::new_concurrent(vec![lang_id]);
                    bundle.add_resource(FluentResource::try_new(#resource_ident.to_string()).unwrap()).unwrap();
                    bundle
                });
            }
        }).collect()
}

pub fn language_bundle_lookup_function_definition(
    default_language: &str,
    bundle: &MessageBundle,
) -> Result<TokenStream, Error> {
    let language_bundle_mapping = bundle
        .language_literals()
        .into_iter()
        .zip(bundle.langs.iter())
        .map(|(literal, language_bundle)| {
            let resource_ident = language_bundle.static_bundle_ident();
            quote! {
                #literal => return &#resource_ident
            }
        });
    let default_bundle = bundle
        .get_language_bundle(default_language)
        .map(|language_bundle| language_bundle.static_bundle_ident())
        .ok_or_else(|| Error::FallbackLanguageNotFound(default_language.to_string()))?;

    Ok(quote! {
        fn get_bundle<'a, 'b>(lang: &'a str) -> &'b FluentBundle<FluentResource> {
            for common_lang in fluent_static::accept_language::intersection(lang, SUPPORTED_LANGUAGES) {
                match common_lang.as_str() {
                    #(#language_bundle_mapping),* ,
                    _ => continue,
                }
            };
            & #default_bundle
        }
    })
}

pub fn format_message_function_definition(fn_name: &Ident) -> TokenStream {
    quote! {
        fn #fn_name<'a, 'b>(lang_id: &str, message_id: &str, args: Option<&'a FluentArgs>) -> Result<Message<'b>, FluentError> {
            let bundle = get_bundle(lang_id.as_ref());
            let msg = bundle.get_message(message_id).expect("Message not found");
            let mut errors = vec![];
            let result = Message::new(bundle.format_pattern(&msg.value().unwrap(), args, &mut errors));
            if errors.is_empty() {
                Ok(result)
            } else {
                Err(errors.into_iter().next().unwrap())
            }
        }
    }
}
