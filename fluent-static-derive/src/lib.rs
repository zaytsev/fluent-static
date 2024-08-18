use std::{
    collections::BTreeSet,
    env, fs,
    path::{Path, PathBuf},
};

use fluent_static_codegen::bundle::{LanguageBundle, MessageBundle};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Attribute, AttributeArgs, DeriveInput, Error as SyntaxError, Ident,
    ItemStruct, Lit, Meta, MetaList, MetaNameValue, NestedMeta, Result as SyntaxResult,
};
use wax::Glob;

#[proc_macro_attribute]
pub fn fluent_bundle(args: TokenStream, input: TokenStream) -> TokenStream {
    let item_struct = parse_macro_input!(input as ItemStruct);
    let args = parse_macro_input!(args as AttributeArgs);

    let result = quote! {};

    TokenStream::from(result)
}

macro_rules! syntax_err {
    ($input:expr, $message:expr $(, $args:expr)*) => {
        ::syn::Result::Err(::syn::Error::new_spanned($input, format!($message $(, $args)*)))
    }
}

struct FluentBundle {
    default_language: String,
    language_bundles: Vec<LanguageBundle>,
}

impl Parse for FluentBundle {
    fn parse(input: ParseStream) -> SyntaxResult<Self> {
        let args: AttributeArgs = input.parse()?;
        todo!()
    }
}

impl FluentBundle {
    fn parse(item_struct: &ItemStruct, args: &AttributeArgs) -> SyntaxResult<Self> {
        let span = args
            .iter()
            .map(|arg| arg.to_token_stream())
            .collect::<TokenStream2>();

        let FluentBundleAttributes {
            glob,
            resources,
            mut default_language,
        } = FluentBundleAttributes::parse(args)?;

        let project_dir = &env::var_os("CARGO_MANIFEST_DIR_OVERRIDE") // used for tests
            .or_else(|| env::var_os("CARGO_MANIFEST_DIR"))
            .ok_or_else(|| {
                syntax_err!(
                    span,
                    "Environment variable 'CARGO_MANIFEST_DIR' is not defined"
                )
            })?;

        let base_path = Path::new(project_dir);

        let mut fluent_resources: Vec<(String, PathBuf)> = resources
            .into_iter()
            .map(|(lang, rel_path)| (lang, base_path.join(rel_path)))
            .collect();

        if let Some(pattern) = glob {
            let mut resources_found = false;

            let glob = Glob::new(&pattern).map_err(|e| {
                syntax_err!(span, "Error parsing include pattern {}: {}", pattern, e)
            })?;

            if glob.captures().count() != 1 {
                return syntax_err!(span, "");
            }

            for entry_result in glob.walk(&base_path) {
                let entry = entry_result.map_err(|e| {
                    input.error(format!(
                        "Error enumerating Fluent resources: {}",
                        e.to_string()
                    ))
                })?;

                let language = entry.matched().get(1).ok_or_else(|| {
                    input.error(format!(
                        "Unable to determine resource {} language",
                        entry.path().to_string_lossy()
                    ))
                })?;

                resources_found = true;

                fluent_resources.push((language.to_string(), entry.into_path()))
            }

            if !resources_found {
                return Err(input.error(format!(
                    "Not Fluent resources matching pattern '{}' found",
                    pattern
                )));
            }
        }

        let mut language_resources = Vec::new();
        let mut default_language_found = false;

        if fluent_resources.is_empty() {
            return Err(input.error("Fluent bundle has no resources"));
        }

        for (language, path) in fluent_resources {
            let resource = fs::read_to_string(path.as_path()).map_err(|e| {
                input.error(format!(
                    "Error reading Fluent resource {}: {}",
                    path.to_string_lossy(),
                    e
                ))
            })?;

            if default_language.is_none() {
                default_language = Some(language.clone());
                default_language_found = true;
            } else if !default_language_found {
                default_language_found = default_language.as_ref() == Some(&language);
            }

            language_resources.push((language, resource));
        }

        if !default_language_found {
            return Err(input.error(format!(
                "Fluent bundle resources doesn't contain default language '{}'",
                default_language.unwrap_or_default()
            )));
        }

        let bundle_path = ""; // TODO is bundle path really needed?
        let message_bundle =
            MessageBundle::create(&name.to_string(), bundle_path, language_resources)
                .map_err(|e| input.error(describe_error(e)))?;

        let default_language = default_language.unwrap_or_default();

        Ok(FluentBundleSpec {
            name,
            message_bundle,
            default_language,
        })
    }
}

struct FluentBundleAttributes {
    glob: Option<String>,
    resources: Vec<(String, String)>,
    default_language: Option<String>,
}

impl FluentBundleAttributes {
    fn parse(args: &AttributeArgs) -> SyntaxResult<FluentBundleAttributes> {
        let mut glob = None;
        let mut resources = Vec::new();
        let mut default_language = None;

        for arg in args {
            match arg {
                NestedMeta::Meta(meta) => match meta {
                    Meta::Path(_) => Err(SyntaxError::new_spanned(
                        arg,
                        "Unexpected argument", // TODO improve error description
                    )),
                    Meta::List(list) => read_resource(arg, list, &mut resources),
                    Meta::NameValue(nv) => read_default_language(arg, nv, &mut default_language),
                },
                NestedMeta::Lit(lit) => read_resource_glob(arg, lit, &mut glob),
            }?;
        }

        Ok(FluentBundleAttributes {
            glob,
            resources,
            default_language,
        })
    }
}

fn read_resource_glob(
    attr: &impl ToTokens,
    lit: &Lit,
    glob: &mut Option<String>,
) -> SyntaxResult<()> {
    if let Lit::Str(s) = lit {
        if glob.replace(s.value()).is_some() {
            Err(SyntaxError::new_spanned(
                attr,
                "At most one \"string literal\" is allowed",
            ))
        } else {
            Ok(())
        }
    } else {
        Err(SyntaxError::new_spanned(
            attr,
            "Expected string literal, found something else",
        ))
    }
}

fn read_default_language(
    attr: &impl ToTokens,
    nv: &MetaNameValue,
    lang: &mut Option<String>,
) -> SyntaxResult<()> {
    let key = nv
        .path
        .get_ident()
        .ok_or_else(|| SyntaxError::new_spanned(attr, "Unexpected attribute format"))?
        .to_string();
    if let Lit::Str(lit_str) = &nv.lit {
        let value = lit_str.value();

        match key.as_str() {
            "default_language" => {
                if lang.replace(value).is_some() {
                    Err(SyntaxError::new_spanned(
                        attr,
                        "'default_language' is defined multiple times",
                    ))
                } else {
                    Ok(())
                }
            }
            name => Err(SyntaxError::new_spanned(
                attr,
                format!("Unexpected attribute name {}", name),
            )),
        }
    } else {
        Err(SyntaxError::new_spanned(
            attr,
            "Only 'name = \"string literal\" metadata supported",
        ))
    }
}

fn read_resource(
    attr: &impl ToTokens,
    list: &MetaList,
    resources: &mut Vec<(String, String)>,
) -> SyntaxResult<()> {
    if list.path.is_ident("resource") {
        let params: Vec<&NestedMeta> = list.nested.iter().collect();
        if params.len() == 2 {
            let resource = get_resource_param(attr, &params.get(0).unwrap(), "path")?;
            let language = get_resource_param(attr, &params.get(1).unwrap(), "language")?;
            resources.push((language, resource));
            Ok(())
        } else {
            Err(SyntaxError::new_spanned(
                attr,
                "'resource' expected to have two arguments: resource(\"path\", \"language\")",
            ))
        }
    } else {
        Err(SyntaxError::new_spanned(
            attr,
            format!(
                "Expected 'resource' but got '{}' instead",
                list.path
                    .get_ident()
                    .map(|i| i.to_string())
                    .unwrap_or_default(),
            ),
        ))
    }
}

fn get_resource_param(
    attr: &impl ToTokens,
    meta: &NestedMeta,
    param_name: &str,
) -> SyntaxResult<String> {
    if let NestedMeta::Lit(lit) = meta {
        if let Lit::Str(lit_str) = lit {
            return Ok(lit_str.value());
        }
    }

    Err(SyntaxError::new_spanned(
        attr,
        format!("Expected resource {} to be \"string literal\"", param_name),
    ))
}

fn describe_error(error: fluent_static_codegen::Error) -> String {
    match error {
        fluent_static_codegen::Error::MessageBundleValidationError {
            bundle,
            path,
            entries,
        } => {
            let langs = |s: &BTreeSet<String>| {
                s.iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<&str>>()
                    .join(", ")
            };
            entries.into_iter().map(|entry| {
                let defined = langs(&entry.defined_in_languages);
                let undefined = langs(&entry.undefined_in_languages);
                format!("Fluent bundle '{}' {} validation error: Message '{}' is defined for {} but is undefined for {}", 
                        bundle, path, entry.message.name, defined, undefined)
            }).collect::<Vec<String>>().join("\n")
        }
        e => e.to_string(),
    }
}
