use convert_case::{Case, Casing};
use proc_macro2::{Literal, TokenStream};
use syn::Ident;

use crate::{bundle::MessageBundle, message::Message, Error};
use quote::{format_ident, quote};

use super::{common, CodeGenerator};

pub struct FunctionPerMessageCodeGenerator {
    default_language: String,
}

impl CodeGenerator for FunctionPerMessageCodeGenerator {
    fn generate(&self, bundle: &MessageBundle) -> Result<TokenStream, Error> {
        let module_name = bundle.name_ident();
        let supported_languages = bundle.language_literals();
        let language_bundles = common::language_bundle_definitions(bundle);
        let language_bundle_lookup_fn =
            common::language_bundle_lookup_function_definition(&self.default_language, bundle)?;
        let message_format_fn = format_ident!("format_message");
        let format_message_fn = common::format_message_function_definition(&message_format_fn);
        let format_message_functions =
            message_functions_definitions(&self.default_language, bundle, &message_format_fn)?;

        let mut module = quote! {
            pub mod #module_name {
                #[allow(unused_imports)]
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

fn message_function_definition(msg: &Message, delegate_fn: &Ident) -> TokenStream {
    let function_ident = msg.function_ident();
    let message_name_literal = msg.message_name_literal();
    let maybe_attribute_literal = msg.maybe_attribute_name_literal();
    let msg_total_vars = msg.vars().len();
    if msg_total_vars == 0 {
        quote! {
            pub fn #function_ident(lang_id: impl AsRef<str>) -> Message<'static> {
                #delegate_fn(lang_id.as_ref(), #message_name_literal, #maybe_attribute_literal, None)
                    .expect("Not fallible without variables; qed")
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
            pub fn #function_ident<'a>(lang_id: impl AsRef<str>, #(#function_args: impl Into<FluentValue<'a>>),*) -> Result<Message<'static>, FluentError> {
                let mut args = FluentArgs::with_capacity(#capacity);
                #(#fluent_args)*
                #delegate_fn(lang_id.as_ref(), #message_name_literal, #maybe_attribute_literal, Some(&args))
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
        .flat_map(common::expand_message_attributes)
        .map(|msg| message_function_definition(msg, format_message_name))
        .collect())
}

#[cfg(test)]
mod test {
    use proc_macro2::Literal;
    use quote::quote;

    use crate::{bundle::MessageBundle, codegen::CodeGenerator};

    #[test]
    fn codegen() {
        let resource_en = include_str!("../../test/bundle_en.ftl");
        let resource_en_uk = include_str!("../../test/bundle_en-UK.ftl");

        let message_bundle = MessageBundle::create(
            "messages",
            "test/messages.ftl",
            vec![
                ("en".to_string(), resource_en.to_string()),
                ("en-UK".to_string(), resource_en_uk.to_string()),
            ],
        )
        .unwrap();

        let actual = super::FunctionPerMessageCodeGenerator::new("en")
            .generate(&message_bundle)
            .unwrap();

        let resource_en_literal = Literal::string(resource_en);
        let resource_en_uk_literal = Literal::string(resource_en_uk);

        let expected = quote! {
            pub mod test {
                pub mod messages {
                    #[allow(unused_imports)]
                    use fluent_static::fluent_bundle::{FluentBundle, FluentResource, FluentValue, FluentArgs, FluentError};
                    use fluent_static::once_cell::sync::Lazy;
                    use fluent_static::Message;

                    static SUPPORTED_LANGUAGES: &[&str] = &["en", "en-UK"];

                    static EN_RESOURCE: &str = #resource_en_literal;
                    static EN_BUNDLE: Lazy<FluentBundle<FluentResource>> = Lazy::new(|| {
                        let lang_id = fluent_static::unic_langid::langid!("en");
                        let mut bundle: FluentBundle<FluentResource> = FluentBundle::new_concurrent(vec![lang_id]);
                        bundle.add_resource(FluentResource::try_new(EN_RESOURCE.to_string()).unwrap()).unwrap();
                        bundle
                    });

                    static EN_UK_RESOURCE: &str = #resource_en_uk_literal;
                    static EN_UK_BUNDLE: Lazy<FluentBundle<FluentResource>> = Lazy::new(|| {
                        let lang_id = fluent_static::unic_langid::langid!("en-UK");
                        let mut bundle: FluentBundle<FluentResource> = FluentBundle::new_concurrent(vec![lang_id]);
                        bundle.add_resource(FluentResource::try_new(EN_UK_RESOURCE.to_string()).unwrap()).unwrap();
                        bundle
                    });

                    fn get_bundle(lang: &str) -> &'static FluentBundle<FluentResource> {
                        for common_lang in fluent_static::accept_language::intersection(lang, SUPPORTED_LANGUAGES) {
                            match common_lang.as_str() {
                                "en" => return &EN_BUNDLE,
                                "en-UK" => return &EN_UK_BUNDLE,
                                _ => continue,
                            }
                        };
                        & EN_BUNDLE
                    }

                    fn format_message(lang_id: &str, message_id: &str, attr: Option<&str>, args: Option<&FluentArgs>) -> Result<Message<'static>, FluentError> {
                        let bundle = get_bundle(lang_id.as_ref());
                        let msg = bundle.get_message(message_id).expect("Message not found");
                        let pattern = if let Some(attr) = attr {
                            msg.get_attribute(attr).unwrap().value()
                        } else {
                            msg.value().unwrap()
                        };
                        let mut errors = vec![];
                        let result = Message::new(bundle.format_pattern(pattern, args, &mut errors));
                        if errors.is_empty() {
                            Ok(result)
                        } else {
                            Err(errors.into_iter().next().unwrap())
                        }
                    }

                    pub fn test(lang_id: impl AsRef<str>) -> Message<'static> {
                        format_message(lang_id.as_ref(), "test", None, None)
                            .expect("Not fallible without variables; qed")
                    }

                    pub fn test_attr_1(lang_id: impl AsRef<str>) -> Message<'static> {
                        format_message(lang_id.as_ref(), "test", Some("attr1"), None)
                            .expect("Not fallible without variables; qed")
                    }

                    pub fn test_attr_2(lang_id: impl AsRef<str>) -> Message<'static> {
                        format_message(lang_id.as_ref(), "test", Some("attr2"), None)
                            .expect("Not fallible without variables; qed")
                    }

                    pub fn test_args_1<'a>(lang_id: impl AsRef<str>, name: impl Into<FluentValue<'a>>) -> Result<Message<'static>, FluentError> {
                        let mut args = FluentArgs::with_capacity(1);
                        args.set("name", name);
                        format_message(lang_id.as_ref(), "test-args1", None, Some(&args))
                    }

                    pub fn test_order<'a>(lang_id: impl AsRef<str>, zzz: impl Into<FluentValue<'a>>, aaa: impl Into<FluentValue<'a>>) -> Result<Message<'static>, FluentError> {
                        let mut args = FluentArgs::with_capacity(2);
                        args.set("zzz", zzz);
                        args.set("aaa", aaa);
                        format_message(lang_id.as_ref(), "test-order", None, Some(&args))
                    }
                }
            }
        };

        assert_eq!(expected.to_string(), actual.to_string());
    }
}
