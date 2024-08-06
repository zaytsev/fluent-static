use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use convert_case::{Case, Casing};
use fluent_syntax::ast;
use proc_macro2::Literal;
use quote::format_ident;
use syn::Ident;

use crate::{
    error::MessageValidationErrorEntry,
    message::{Message, NormalizedMessage},
    Error,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageBundle {
    pub name: String,
    pub path: PathBuf,
    pub language_bundles: Vec<LanguageBundle>,
}

impl MessageBundle {
    pub fn create(
        name: &str,
        path: impl AsRef<Path>,
        language_resource: Vec<(String, String)>,
    ) -> Result<Self, Error> {
        let bundles: Vec<LanguageBundle> = language_resource
            .into_iter()
            .map(|(language, resource)| LanguageBundle::create(language, resource))
            .collect::<Result<Vec<LanguageBundle>, Error>>()?;

        Self {
            name: name.to_string(),
            path: path.as_ref().to_path_buf(),
            language_bundles: bundles,
        }
        .validate()
    }

    fn validate(self) -> Result<Self, Error> {
        let all_langs = self
            .language_bundles
            .iter()
            .map(|bundle| bundle.language.as_str())
            .collect::<HashSet<&str>>();

        let validation_errors = self
            .language_bundles
            .iter()
            .flat_map(|bundle| {
                bundle
                    .messages()
                    .into_iter()
                    .map(|msg| (bundle.language.as_str(), msg.normalize()))
            })
            .fold(
                HashMap::<NormalizedMessage, HashSet<&str>>::new(),
                |mut acc, (lang, msg)| {
                    acc.entry(msg).or_insert_with(HashSet::new).insert(lang);
                    acc
                },
            )
            .into_iter()
            .filter_map(|(message, langs)| {
                if langs.len() != self.language_bundles.len() {
                    let undefined = all_langs
                        .difference(&langs)
                        .map(|lang| lang.to_string())
                        .collect();
                    let defined = langs.into_iter().map(String::from).collect();
                    Some(MessageValidationErrorEntry {
                        message,
                        defined_in_languages: defined,
                        undefined_in_languages: undefined,
                    })
                } else {
                    None
                }
            })
            .collect::<Vec<MessageValidationErrorEntry>>();

        if !validation_errors.is_empty() {
            Err(Error::MessageBundleValidationError {
                bundle: self.name,
                entries: validation_errors,
                path: self.path.to_string_lossy().to_string(),
            })
        } else {
            Ok(self)
        }
    }

    pub fn get_language_bundle(&self, lang: &str) -> Option<&LanguageBundle> {
        self.language_bundles
            .iter()
            .find(|bundle| bundle.language() == lang)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn name_ident(&self) -> Ident {
        format_ident!("{}", self.name.to_case(Case::Snake))
    }

    pub fn language_literals(&self) -> Vec<Literal> {
        self.language_bundles
            .iter()
            .map(|lang| Literal::string(lang.language()))
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageBundle {
    pub(crate) language: String,
    pub(crate) resource: String,
    pub(crate) messages: Vec<Message>,
}

impl LanguageBundle {
    pub fn create(language: String, resource: String) -> Result<Self, Error> {
        let messages = Self::parse(&resource)?;
        Ok(Self {
            language,
            resource,
            messages,
        })
    }

    fn parse(content: &str) -> Result<Vec<Message>, Error> {
        let ast = fluent_syntax::parser::parse(content)
            .map_err(|(_, errors)| Error::FluentParserError { errors })?;

        ast.body
            .iter()
            .filter_map(|entry| {
                if let ast::Entry::Message(message) = entry {
                    Some(message)
                } else {
                    None
                }
            })
            .map(|message| Message::parse(message))
            .collect()
    }

    pub fn messages(&self) -> Vec<&Message> {
        self.messages.iter().collect()
    }

    pub fn language(&self) -> &str {
        &self.language
    }

    pub fn resource(&self) -> &str {
        &self.resource
    }

    pub fn resource_literal(&self) -> Literal {
        Literal::string(self.resource())
    }

    pub fn static_bundle_ident(&self) -> Ident {
        format_ident!("{}_BUNDLE", self.language.to_case(Case::ScreamingSnake))
    }

    pub fn static_resource_ident(&self) -> Ident {
        format_ident!("{}_RESOURCE", self.language.to_case(Case::ScreamingSnake))
    }
}
