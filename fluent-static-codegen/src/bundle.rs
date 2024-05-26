use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

use convert_case::{Case, Casing};
use fluent_syntax::ast;
use proc_macro2::Literal;
use quote::format_ident;
use syn::Ident;

use crate::{message::Message, Error};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageBundle {
    pub name: String,
    pub path: PathBuf,
    pub langs: Vec<LanguageBundle>,
}

impl MessageBundle {
    pub fn create(
        name: &str,
        path: impl AsRef<Path>,
        language_resource: Vec<(String, String)>,
    ) -> Result<Self, Error> {
        let langs = language_resource
            .into_iter()
            .map(|(lang, resource)| LanguageBundle::create(lang, resource))
            .collect::<Result<Vec<LanguageBundle>, Error>>()?;
        Self {
            name: name.to_string(),
            path: path.as_ref().to_path_buf(),
            langs,
        }
        .validate()
    }

    pub fn get_language_bundle(&self, lang: &str) -> Option<&LanguageBundle> {
        self.langs.iter().find(|bundle| bundle.language() == lang)
    }

    fn validate(self) -> Result<Self, Error> {
        let mut result = vec![];
        for i in 0..self.langs.len() {
            for j in i + 1..self.langs.len() {
                let this = &self.langs[i];
                let that = &self.langs[j];
                let diff = this.diff(&that);
                if !diff.is_empty() {
                    let names: Vec<String> =
                        diff.into_iter().map(|msg| msg.name().to_string()).collect();
                    result.push((
                        this.language().to_string(),
                        that.language.to_string(),
                        names,
                    ));
                }
            }
        }
        if !result.is_empty() {
            Err(Error::MessageBundleValidationError {
                bundle: self.path.to_string_lossy().to_string(),
                mismatching_messages: result,
            })
        } else {
            Ok(self)
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn name_ident(&self) -> Ident {
        format_ident!("{}", self.name.to_case(Case::Snake))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageBundle {
    pub(crate) language: String,
    pub(crate) resource: String,
    pub(crate) messages: BTreeSet<Message>,
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

    fn parse(content: &str) -> Result<BTreeSet<Message>, Error> {
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
            .map(|message| Message::parse(message))
            .collect()
    }

    pub fn messages(&self) -> BTreeSet<&Message> {
        self.messages.iter().collect()
    }
    pub fn diff<'a>(&'a self, other: &'a LanguageBundle) -> Vec<&'a Message> {
        self.messages.difference(&other.messages).collect()
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
