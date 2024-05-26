use convert_case::{Case, Casing};
use proc_macro2::{Literal, TokenStream};
use syn::Ident;

use crate::{bundle::MessageBundle, message::Message, Error};
use quote::{format_ident, quote};

use super::CodeGenerator;

pub struct FunctionPerMessageCodeGenerator {
    default_language: String,
}

impl CodeGenerator for FunctionPerMessageCodeGenerator {
    fn generate(&self, bundle: &MessageBundle) -> Result<TokenStream, Error> {
        let module_name = bundle.name_ident();
        let supported_languages = supported_lanuages_literals(bundle);
        let language_bundles = language_bundle_definitions(bundle);
        let language_bundle_lookup_fn =
            language_bundle_lookup_function_definition(&self.default_language, bundle)?;
        let (format_message_fn_ident, format_message_fn) = format_message_function_definition();
        let format_message_functions = message_functions_definitions(
            &self.default_language,
            bundle,
            &format_message_fn_ident,
        )?;

        let mut module = quote! {
            pub mod #module_name {
                use fluent_static::fluent_bundle::{FluentBundle, FluentResource, FluentValue, FluentArgs, FluentError};
                use fluent_static::once_cell::sync::Lazy;
                use fluent_static::Message;

                static SUPPORTED_LANGUAGES: &[&str] = &[#(#supported_languages),*];
                #(#language_bundles)*
                #language_bundle_lookup_fn
                #format_message_fn

                #(#format_message_functions)*
            }
        };

        let mut parent_path = bundle.path().parent();
        while let Some(parent) = parent_path {
            if let Some(dir_name) = parent.file_name() {
                let name = dir_name
                    .to_str()
                    .ok_or_else(|| Error::InvalidPath(bundle.path().to_path_buf()))?;
                let parent_module_name = format_ident!("{}", name.to_case(Case::Snake));
                module = quote! {
                    pub mod #parent_module_name {
                        #module
                    }
                }
            }
            parent_path = parent.parent();
        }

        Ok(module)
    }
}

impl FunctionPerMessageCodeGenerator {
    pub fn new(default_language: &str) -> Self {
        Self {
            default_language: default_language.to_string(),
        }
    }
}

fn supported_lanuages_literals(bundle: &MessageBundle) -> Vec<Literal> {
    bundle
        .langs
        .iter()
        .map(|lang| Literal::string(lang.language()))
        .collect()
}

fn language_bundle_definitions(bundle: &MessageBundle) -> Vec<TokenStream> {
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

fn language_bundle_lookup_function_definition(
    default_language: &str,
    bundle: &MessageBundle,
) -> Result<TokenStream, Error> {
    let language_bundle_mapping = supported_lanuages_literals(bundle)
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

fn format_message_function_definition() -> (Ident, TokenStream) {
    let function_name_ident = format_ident!("format_message");
    let tokens = quote! {
        fn #function_name_ident<'a, 'b>(lang_id: &str, message_id: &str, args: Option<&'a FluentArgs>) -> Result<Message<'b>, FluentError> {
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
    };

    (function_name_ident, tokens)
}

fn message_function_definition(msg: &Message, delegate_fn: &Ident) -> TokenStream {
    let function_ident = msg.function_ident();
    let message_name_literal = msg.message_name_literal();
    let msg_total_vars = msg.vars().len();
    if msg_total_vars == 0 {
        quote! {
            pub fn #function_ident<'b>(lang_id: impl AsRef<str>) -> Result<Message<'b>, FluentError> {
                #delegate_fn(lang_id.as_ref(), #message_name_literal, None)
            }
        }
    } else {
        let (function_args, fluent_args): (Vec<Ident>, Vec<TokenStream>) = msg
            .vars()
            .into_iter()
            .map(|var| {
                let name = var.literal();
                let ident = var.ident();
                (
                    ident.clone(),
                    quote! {
                        args.set(#name, #ident);
                    },
                )
            })
            .unzip();
        let capacity = Literal::usize_unsuffixed(msg_total_vars);
        quote! {
            pub fn #function_ident<'a, 'b>(lang_id: impl AsRef<str>, #(#function_args: impl Into<FluentValue<'a>>),*) -> Result<Message<'b>, FluentError> {
                let mut args = FluentArgs::with_capacity(#capacity);
                #(#fluent_args)*
                #delegate_fn(lang_id.as_ref(), #message_name_literal, Some(&args))
            }
        }
    }
}

fn message_functions_definitions(
    default_language: &str,
    bundle: &MessageBundle,
    format_message_name: &Ident,
) -> Result<Vec<TokenStream>, Error> {
    let language_bundle = bundle
        .get_language_bundle(default_language)
        .ok_or_else(|| Error::FallbackLanguageNotFound(default_language.to_string()))?;

    Ok(language_bundle
        .messages()
        .into_iter()
        .map(|msg| message_function_definition(msg, format_message_name))
        .collect())
}

#[cfg(test)]
mod test {
    use quote::quote;

    use crate::{bundle::MessageBundle, codegen::CodeGenerator};

    #[test]
    fn codegen() {
        let message_bundle = MessageBundle::create(
            "messages",
            "test/messages.ftl",
            vec![
                (
                    "en".to_string(),
                    "test=Test message\ntest-args1=Hello { $name }".to_string(),
                ),
                (
                    "en-UK".to_string(),
                    "test=UK message\ntest-args1=Greetings { $name }".to_string(),
                ),
            ],
        )
        .unwrap();

        let actual = super::FunctionPerMessageCodeGenerator::new("en")
            .generate(&message_bundle)
            .unwrap();

        let expected = quote! {
            pub mod test {
                pub mod messages {
                    use fluent_static::fluent_bundle::{FluentBundle, FluentResource, FluentValue, FluentArgs, FluentError};
                    use fluent_static::once_cell::sync::Lazy;
                    use fluent_static::Message;

                    static SUPPORTED_LANGUAGES: &[&str] = &["en", "en-UK"];

                    static EN_RESOURCE: &str = "test=Test message\ntest-args1=Hello { $name }";
                    static EN_BUNDLE: Lazy<FluentBundle<FluentResource>> = Lazy::new(|| {
                        let lang_id = fluent_static::unic_langid::langid!("en");
                        let mut bundle: FluentBundle<FluentResource> = FluentBundle::new_concurrent(vec![lang_id]);
                        bundle.add_resource(FluentResource::try_new(EN_RESOURCE.to_string()).unwrap()).unwrap();
                        bundle
                    });

                    static EN_UK_RESOURCE: &str = "test=UK message\ntest-args1=Greetings { $name }";
                    static EN_UK_BUNDLE: Lazy<FluentBundle<FluentResource>> = Lazy::new(|| {
                        let lang_id = fluent_static::unic_langid::langid!("en-UK");
                        let mut bundle: FluentBundle<FluentResource> = FluentBundle::new_concurrent(vec![lang_id]);
                        bundle.add_resource(FluentResource::try_new(EN_UK_RESOURCE.to_string()).unwrap()).unwrap();
                        bundle
                    });

                    fn get_bundle<'a, 'b>(lang: &'a str) -> &'b FluentBundle<FluentResource> {
                        for common_lang in fluent_static::accept_language::intersection(lang, SUPPORTED_LANGUAGES) {
                            match common_lang.as_str() {
                                "en" => return &EN_BUNDLE,
                                "en-UK" => return &EN_UK_BUNDLE,
                                _ => continue,
                            }
                        };
                        & EN_BUNDLE
                    }

                    fn format_message<'a, 'b>(lang_id: &str, message_id: &str, args: Option<&'a FluentArgs>) -> Result<Message<'b>, FluentError> {
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

                    pub fn test<'b>(lang_id: impl AsRef<str>) -> Result<Message<'b>, FluentError> {
                        format_message(lang_id.as_ref(), "test", None)
                    }

                    pub fn test_args_1<'a, 'b>(lang_id: impl AsRef<str>, name: impl Into<FluentValue<'a>>) -> Result<Message<'b>, FluentError> {
                        let mut args = FluentArgs::with_capacity(1);
                        args.set("name", name);
                        format_message(lang_id.as_ref(), "test-args1", Some(&args))
                    }
                }
            }
        };

        assert_eq!(expected.to_string(), actual.to_string());
    }
}
