use convert_case::{Case, Casing};
use fluent_syntax::ast::{self, Expression, InlineExpression, Message, Pattern, PatternElement};
use proc_macro2::{Literal, TokenStream};
use quote::{format_ident, quote};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};
use syn::Ident;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    PathPrefixError(#[from] std::path::StripPrefixError),

    #[error("Path contans invalid symbols")]
    InvalidPath,

    #[error("Error parsing fluent resource")]
    FluentParserError {
        errors: Vec<fluent_syntax::parser::ParserError>,
    },

    #[error("Found unsupported feature {feature}: {id}")]
    UnsupportedFeature { feature: String, id: String },
}

#[derive(Debug, Clone)]
struct MessageBundle {
    name: String,
    language_resources: Vec<(String, String)>,
    prefix: Vec<String>,
}

pub fn generate(root_dir: impl AsRef<Path>, fallback_language: &str) -> Result<String, Error> {
    let resource_paths = fluent_resources(&root_dir)?;

    let mut resources = vec![];
    for res_path in resource_paths {
        println!("cargo:rerun-if-changed={}", res_path.to_string_lossy());
        let path = res_path.strip_prefix(root_dir.as_ref())?.into();
        let content = fs::read_to_string(res_path)?;
        resources.push((path, content));
    }
    let result = generate_from_resources(resources, fallback_language)?;

    Ok(result.to_string())
}

fn generate_from_resources(
    resources: Vec<(PathBuf, String)>,
    fallback_language: &str,
) -> Result<TokenStream, Error> {
    let bundles = create_message_bundles(resources)?;

    let mut modules = vec![];
    for bundle in bundles {
        let module_ident = format_ident!("{}", bundle.name);

        let supported_languages = bundle
            .language_resources
            .iter()
            .map(|(lang, _)| Literal::string(lang));

        let bundle_definitions = bundle
            .language_resources
            .iter()
            .map(|(lang, content)| language_bundles(lang, content))
            .collect::<Vec<TokenStream>>();

        let language_bundle_mapping = bundle.language_resources.iter().map(|(lang, _)| {
            let lang_litreral = Literal::string(lang);
            let bundle = fluent_bundle_name(lang);
            quote! {
                #lang_litreral => return &#bundle
            }
        });

        let default_bundle = fluent_bundle_name(fallback_language);

        let fns = if let Some(fallback_content) =
            bundle
                .language_resources
                .iter()
                .find_map(|(lang, content)| {
                    if fallback_language == lang {
                        Some(content)
                    } else {
                        None
                    }
                }) {
            generate_messages(fallback_language, fallback_content.as_str())?
        } else {
            vec![]
        };

        let mut module = quote! {
            pub mod #module_ident {
                use std::borrow::Cow;

                use fluent_static::fluent_bundle::{FluentBundle, FluentResource, FluentValue, FluentArgs};
                use fluent_static::once_cell::sync::Lazy;

                static SUPPORTED_LANGUAGES: &[&str] = &[#(#supported_languages),*];

                #(#bundle_definitions)*

                fn get_bundle<'a, 'b>(lang: &'a str) -> &'b FluentBundle<FluentResource> {
                    for common_lang in fluent_static::accept_language::intersection(lang, SUPPORTED_LANGUAGES) {
                        match common_lang.as_str() {
                            #(#language_bundle_mapping),* ,
                            _ => continue,
                        }
                    }
                    & #default_bundle
                }

                #(#fns)*
            }
        };

        for prefix in bundle.prefix.iter().rev() {
            module = quote! {
              pub mod #prefix {
                  #module
              }
            };
        }

        modules.push(module);
    }

    let result = quote! {
      #(#modules)*
    };

    Ok(result)
}

fn generate_messages(_lang: &str, content: &str) -> Result<Vec<TokenStream>, Error> {
    let resource = fluent_syntax::parser::parse(content)
        .map_err(|(_, errors)| Error::FluentParserError { errors })?;

    resource
        .body
        .iter()
        .filter_map(|entry| {
            if let ast::Entry::Message(message) = entry {
                Some(message)
            } else {
                None
            }
        })
        .map(|message| message_function(message))
        .collect()
}

fn message_function<T: AsRef<str> + std::fmt::Debug>(
    message: &Message<T>,
) -> Result<TokenStream, Error> {
    let message_id = message.id.name.as_ref();
    let function_name = format_ident!("{}", message_id.to_case(Case::Snake));
    let message_id_literal = Literal::string(message_id);

    let mut fn_args: Vec<Ident> = vec![];
    let mut msg_args: Vec<Literal> = vec![];

    for variable in extract_variables(message.value.as_ref())? {
        msg_args.push(Literal::string(variable.as_str()));
        fn_args.push(format_ident!("{}", variable.to_case(Case::Snake)));
    }

    let message_args = if msg_args.is_empty() {
        quote! {
            let message_args = None;
        }
    } else {
        let total_args = Literal::usize_unsuffixed(msg_args.len());
        let mut args = vec![];
        for (name, val) in msg_args.iter().zip(fn_args.iter()) {
            args.push(quote! {
                args.set(#name, #val);
            });
        }
        quote! {
            let mut args = FluentArgs::with_capacity(#total_args);
            #(#args)*
            let message_args = Some(&args);
        }
    };

    Ok(quote! {
        pub fn #function_name<'a, 'b>(lang_id: impl AsRef<str>, #(#fn_args: impl Into<FluentValue<'a>>),*) -> Cow<'b, str> {
            let bundle = get_bundle(lang_id.as_ref());
            let msg = bundle.get_message(#message_id_literal).unwrap();
            let mut errors = vec![];
            #message_args
            bundle.format_pattern(&msg.value().unwrap(), message_args, &mut errors)
        }
    })
}

fn extract_variables<T: AsRef<str> + std::fmt::Debug>(
    value: Option<&Pattern<T>>,
) -> Result<Vec<String>, Error> {
    let mut result = vec![];
    if let Some(pattern) = value {
        for element in pattern.elements.iter() {
            if let PatternElement::Placeable { expression } = element {
                match expression {
                    Expression::Select { selector, variants } => {
                        if let Some(var) = parse_expression(selector)? {
                            result.push(var)
                        }
                        for variant in variants {
                            result.append(&mut extract_variables(Some(&variant.value))?)
                        }
                    }
                    Expression::Inline(e) => {
                        if let Some(var) = parse_expression(e)? {
                            result.push(var)
                        }
                    }
                }
            }
        }
    }
    Ok(result)
}

fn parse_expression<T: AsRef<str> + std::fmt::Debug>(
    inline_expression: &InlineExpression<T>,
) -> Result<Option<String>, Error> {
    match inline_expression {
        ast::InlineExpression::StringLiteral { .. } => Ok(None),
        ast::InlineExpression::NumberLiteral { .. } => Ok(None),
        ast::InlineExpression::FunctionReference { id, .. } => Err(Error::UnsupportedFeature {
            feature: "function reference".to_string(),
            id: id.name.as_ref().to_string(),
        }),
        ast::InlineExpression::MessageReference { id, .. } => Err(Error::UnsupportedFeature {
            feature: "message reference".to_string(),
            id: id.name.as_ref().to_string(),
        }),
        ast::InlineExpression::TermReference { id, .. } => Err(Error::UnsupportedFeature {
            feature: "term reference".to_string(),
            id: id.name.as_ref().to_string(),
        }),

        ast::InlineExpression::VariableReference { id } => Ok(Some(id.name.as_ref().to_string())),
        ast::InlineExpression::Placeable { expression } => Err(Error::UnsupportedFeature {
            feature: "nested expression".to_string(),
            id: format!("{:?}", *expression),
        }),
    }
}

fn fluent_bundle_name(lang: &str) -> Ident {
    format_ident!("{}_BUNDLE", lang.to_case(Case::ScreamingSnake))
}

fn fluent_resource_name(lang: &str) -> Ident {
    format_ident!("{}_RESOURCE", lang.to_case(Case::ScreamingSnake))
}

fn create_message_bundles(resources: Vec<(PathBuf, String)>) -> Result<Vec<MessageBundle>, Error> {
    let mut bundles_by_path: BTreeMap<String, MessageBundle> = BTreeMap::new();
    for (path, content) in resources {
        if let Some(bundle_name) = path.file_stem() {
            let path_parts = path
                .iter()
                .map(|i| i.to_str().ok_or(Error::InvalidPath).map(|s| s.to_string()))
                .collect::<Result<Vec<String>, Error>>()?;

            if path_parts.len() > 1 {
                let bundle_path = path_parts[1..].join("/");
                let lang = path_parts[0].clone();

                let bundle = if bundles_by_path.contains_key(&bundle_path) {
                    bundles_by_path.get_mut(&bundle_path)
                } else {
                    let prefix = path_parts[1..path_parts.len() - 1].to_vec();
                    let new_bundle = MessageBundle {
                        name: bundle_name.to_string_lossy().into_owned(),
                        language_resources: vec![],
                        prefix,
                    };
                    bundles_by_path.insert(bundle_path.clone(), new_bundle);
                    bundles_by_path.get_mut(&bundle_path)
                }
                .unwrap();

                bundle.language_resources.push((lang, content));
            }
        }
    }

    Ok(bundles_by_path.into_values().collect())
}

fn language_bundles(lang: &String, content: &String) -> TokenStream {
    let lang_id = lang.as_str();
    let resource_ident = fluent_resource_name(lang_id);
    let bundle_ident = fluent_bundle_name(lang_id);
    let resource = Literal::string(content);
    quote! {
        static #resource_ident: &str = #resource;
        static #bundle_ident: Lazy<FluentBundle<FluentResource>> = Lazy::new(|| {
            let lang_id = fluent_static::unic_langid::langid!(#lang_id);
            let mut bundle: FluentBundle<FluentResource> = FluentBundle::new_concurrent(vec![lang_id]);
            bundle.add_resource(FluentResource::try_new(#resource_ident.to_string()).unwrap()).unwrap();
            bundle
        });
    }
}

fn fluent_resources(root_dir: &impl AsRef<Path>) -> Result<Vec<PathBuf>, Error> {
    let mut pending: Vec<PathBuf> = vec![root_dir.as_ref().into()];
    let mut resource_paths: Vec<PathBuf> = vec![];
    while let Some(dir) = pending.pop() {
        for entry in fs::read_dir(dir)? {
            let entry_path = entry?.path();
            if entry_path.is_dir() {
                pending.push(entry_path);
            } else if entry_path
                .extension()
                .is_some_and(|ext| ext.to_string_lossy() == "ftl")
            {
                resource_paths.push(entry_path);
            }
        }
    }
    resource_paths.sort();
    Ok(resource_paths)
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, str::FromStr};

    #[test]
    fn test_message_bundles() {
        let resources = vec![
            (
                PathBuf::from_str("en/main.ftl").unwrap(),
                r#"
                    hello=Hello ${ name }
                "#
                .to_string(),
            ),
            (
                PathBuf::from_str("ru_RU/main.ftl").unwrap(),
                r#"
                    hello=Привет, ${ name }
                "#
                .to_string(),
            ),
            (
                PathBuf::from_str("en/extra/test.ftl").unwrap(),
                r#"
                    hello=Hello ${ name }
                "#
                .to_string(),
            ),
            (
                PathBuf::from_str("ru_RU/extra/test.ftl").unwrap(),
                r#"
                    hello=Привет, ${ name }
                "#
                .to_string(),
            ),
        ];

        let actual = super::create_message_bundles(resources).unwrap();

        assert_eq!(2, actual.len());
        assert_eq!(2, actual[0].language_resources.len());
    }
}
