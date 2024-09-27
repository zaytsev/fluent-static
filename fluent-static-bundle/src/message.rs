use std::{
    collections::{BTreeMap, BTreeSet}, path::{Path, PathBuf}, str::FromStr
};

use convert_case::{Case, Casing};
use fluent_syntax::parser;
use proc_macro2::{Ident, Literal, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use unic_langid::LanguageIdentifier;

use crate::{
    ast::Visitor,
    language::LanguageBuilder,
    types::{FluentMessage, PublicFluentId},
    Error,
};

pub struct MessageBundleBuilder {
    bundle_name: String,
    default_language: Option<LanguageIdentifier>,
    base_dir: Option<PathBuf>,
    language_bundles: BTreeMap<LanguageIdentifier, LanguageBuilder>,
    language_idents: BTreeMap<LanguageIdentifier, Ident>,
    language_bundles_code: Vec<TokenStream2>,
}

impl MessageBundleBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            bundle_name: name.to_string(),
            default_language: None,
            base_dir: None,
            language_idents: BTreeMap::new(),
            language_bundles: BTreeMap::new(),
            language_bundles_code: Vec::new(),
        }
    }

    pub fn set_name(mut self, name: &str) -> Self {
        self.bundle_name = name.to_string();
        self
    }

    pub fn with_default_language(mut self, language_id: &str) -> Result<Self, Error> {
        self.default_language = Some(LanguageIdentifier::from_str(language_id)?);
        Ok(self)
    }

    pub fn with_base_dir(mut self, base_dir: impl AsRef<Path>) -> Self {
        self.base_dir = Some(base_dir.as_ref().to_path_buf());
        self
    }

    fn default_language(&self) -> &LanguageIdentifier {
        self.default_language
            .as_ref()
            .or_else(|| self.language_idents.first_key_value().map(|(k, _)| k))
            .unwrap()
    }

    pub fn add_resource(
        mut self,
        lang_id: &str,
        path: impl AsRef<Path>,
    ) -> Result<Self, crate::Error> {
        let resource_path = if path.as_ref().is_absolute() {
            path.as_ref().to_path_buf()
        } else if let Some(base_dir) = self.base_dir.as_ref() {
            base_dir.join(path)
        } else {
            return Err(Error::UnexpectedRelativePath(path.as_ref().to_path_buf()));
        };

        let language_id = LanguageIdentifier::from_str(lang_id)?;

        let language_ident = format_ident!("Lang{}", language_id.to_string().to_case(Case::Pascal));

        self.language_idents
            .insert(language_id.clone(), language_ident);

        let src =
            std::fs::read_to_string(&resource_path).map_err(|e| Error::ResourceReadError {
                path: resource_path.clone(),
                source: e,
            })?;

        let ast =
            parser::parse(src).map_err(|(_, errors)| crate::Error::FluentResourceParseError {
                errors,
                path: resource_path,
            })?;

        let lang_bundle = self
            .language_bundles
            .entry(language_id)
            .or_insert_with_key(|lang_id| LanguageBuilder::new(lang_id));

        self.language_bundles_code
            .push(lang_bundle.visit_resource(&ast)?);

        Ok(self)
    }

    fn validate(&self) -> Result<&Self, crate::Error> {
        let supported_languages: BTreeSet<&LanguageIdentifier> =
            self.language_bundles.keys().collect();

        if let Some(default_language) = self.default_language.as_ref() {
            if !supported_languages.contains(default_language) {
                return Err(Error::UnsupportedDefaultLanguage {
                    lang: default_language.clone().to_string(),
                });
            }
        }

        let validation_errors: Vec<crate::error::MessageValidationErrorEntry> = self
            .language_bundles
            .iter()
            .fold(BTreeMap::new(), |mut msg_fns, (lang, language_bundle)| {
                // construct a map of message id -> to a set of language ids
                language_bundle
                    .registered_message_fns
                    .iter()
                    .for_each(|(id, _)| {
                        msg_fns
                            .entry(id)
                            .or_insert_with(BTreeSet::new)
                            .insert(lang);
                    });
                msg_fns
            })
            .iter()
            .filter_map(|(id, message_languages)| {
                // check if message is defined for all of the supported languages
                // and if not then report
                if message_languages.len() != supported_languages.len() {
                    let missing_langs = supported_languages
                        .difference(&message_languages)
                        .map(|lang| lang.to_string())
                        .collect();

                    Some(crate::error::MessageValidationErrorEntry {
                        message_id: id.to_string(),
                        defined_in_languages: message_languages
                            .into_iter()
                            .map(|lang| lang.to_string())
                            .collect(),
                        undefined_in_languages: missing_langs,
                    })
                } else {
                    None
                }
            })
            .collect();

        if !validation_errors.is_empty() {
            Err(crate::Error::MessageBundleValidationError {
                bundle: self.bundle_name.clone(),
                path: None,
                entries: validation_errors,
            })
        } else {
            Ok(self)
        }
    }

    fn generate(&self) -> Result<TokenStream2, Error> {
        let formatted_bundle_name = self.bundle_name.to_case(Case::Pascal);
        let bundle_ident = format_ident!("{}", &formatted_bundle_name);

        let (bundle_languages_enum, bundle_languages_code) =
            self.generate_languages_enum(&formatted_bundle_name);

        let language_bundles_code = &self.language_bundles_code;
        let message_fns = self.generate_message_fns(&bundle_languages_enum);
        let default_language_literal = Literal::string(&self.default_language().to_string());

        // impl ::fluent_static::LanguageAware for self::#bundle_ident {
        // }

        Ok(quote! {
            #bundle_languages_code

            #[derive(Debug, Clone)]
            pub struct #bundle_ident {
                language: self::#bundle_languages_enum,
            }

            impl ::fluent_static::LanguageAware for self::#bundle_ident {
                fn language_id(&self) -> &str {
                    self.language.language_id()
                }
            }


            impl ::fluent_static::MessageBundle for self::#bundle_ident {
                fn get(language_id: &str) -> Option<Self> {
                    self::#bundle_languages_enum::get(language_id).map(|language| Self { language })
                }

                fn default_language_id() -> &'static str {
                    #default_language_literal
                }

                fn supported_language_ids() -> &'static [&'static str] {
                    self::#bundle_languages_enum::language_ids()
                }
            }

            impl ::core::default::Default for self::#bundle_ident {
                fn default() -> Self {
                    Self {
                        language: self::#bundle_languages_enum::default(),
                    }
                }
            }

            impl #bundle_ident {
                #(#message_fns)*
                #(#language_bundles_code)*
            }
        })
    }

    fn generate_languages_enum(&self, bundle_name: &str) -> (Ident, TokenStream2) {
        let bundle_languages_enum_ident = format_ident!("{}BundleLanguage", bundle_name);

        let language_idents: Vec<(Literal, &Ident)> = self
            .language_idents
            .iter()
            .map(|(lang_id, ident)| (Literal::string(&lang_id.to_string()), ident))
            .collect();

        let default_lang_ident = self
            .language_idents
            .get(self.default_language())
            .expect("Unable to get default language");

        let language_mappings: Vec<TokenStream2> = language_idents
            .iter()
            .map(|(lang_id, ident)| {
                quote! {
                    #lang_id => Some(Self::#ident)
                }
            })
            .collect();

        let ident_mappings: Vec<TokenStream2> = language_idents
            .iter()
            .map(|(lang_id, ident)| {
                quote! {
                    Self::#ident => #lang_id
                }
            })
            .collect();

        let plural_rules_cardinal_mappings: Vec<TokenStream2> = language_idents
            .iter()
            .map(|(lang_id, ident)| {
                quote! {
                    Self::#ident => {
                        static RULES: ::fluent_static::once_cell::sync::Lazy<::fluent_static::intl_pluralrules::PluralRules> =
                            ::fluent_static::once_cell::sync::Lazy::new(|| 
                                ::fluent_static::intl_pluralrules::PluralRules::create(
                                    ::fluent_static::unic_langid::LanguageIdentifier::from_bytes(#lang_id.as_bytes()).unwrap(),
                                    ::fluent_static::intl_pluralrules::PluralRuleType::CARDINAL).unwrap());
                        &RULES
                    }
                }
            })
            .collect();

        let total_langs = Literal::usize_unsuffixed(language_idents.len());

        let (bundle_languages_literals, bundle_languages_enum_members): (
            Vec<Literal>,
            Vec<&Ident>,
        ) = language_idents.into_iter().unzip();

        (
            bundle_languages_enum_ident.clone(),
            quote! {
                #[derive(Debug, Clone)]
                pub enum #bundle_languages_enum_ident {
                    #(#bundle_languages_enum_members),*
                }

                impl #bundle_languages_enum_ident {
                    const LANGUAGE_IDS: [&'static str; #total_langs] = [#(#bundle_languages_literals),*];

                    fn get(lang_id: &str) -> Option<Self> {
                        match lang_id {
                            #(#language_mappings),*,
                            _ => None
                        }
                    }

                    fn language_ids() -> &'static [&'static str] {
                        &Self::LANGUAGE_IDS
                    }

                    fn plural_rules_cardinal(&self) -> &'static ::fluent_static::intl_pluralrules::PluralRules {
                        match self {
                            #(#plural_rules_cardinal_mappings),*
                        }
                    }

                }

                impl ::fluent_static::LanguageAware for self::#bundle_languages_enum_ident {
                    fn language_id(&self) -> &str {
                        match self {
                            #(#ident_mappings),*
                        }
                    }
                }

                impl ::core::default::Default for self::#bundle_languages_enum_ident {
                    fn default() -> Self {
                        Self::#default_lang_ident
                    }
                }
            },
        )
    }

    fn generate_message_fns(&self, languages_enum: &Ident) -> Vec<TokenStream2> {
        self.language_bundles
            .get(self.default_language())
            .iter()
            .flat_map(|bundle| {
                bundle
                    .registered_message_fns
                    .iter()
                    .map(|(id, def)| self.generate_message_fn(languages_enum, id, def))
            })
            .collect()
    }

    fn generate_message_fn(
        &self,
        languages_enum: &Ident,
        msg_fn_id: &PublicFluentId,
        msg: &FluentMessage,
    ) -> TokenStream2 {
        let fn_ident = format_ident!("{}", msg.id().to_string().replace('.', "_").to_case(Case::Snake));

        let vars = msg.declared_vars();
        let fn_generics = if msg.has_vars() {
            quote! {<'a>}
        } else {
            quote! {}
        };
        let var: Vec<&Ident> = vars.iter().map(|var| &var.var_ident).collect();

        let lang_selectors: Vec<TokenStream2> = self
            .language_bundles
            .iter()
            .flat_map(|(lang, bundle)| {
                bundle
                    .registered_message_fns
                    .get(msg_fn_id)
                    .map(|fn_def| (lang, fn_def))
            })
            .map(
                |(
                    lang,
                    lang_msg,
                )| {
                    let lang_fn_ident = lang_msg.fn_ident();
                    let fn_vars: BTreeSet<Ident> = lang_msg.vars().into_iter().map(|var| var.var_ident).collect();
                    let lang = self.language_idents.get(lang).expect("Unexpected language");
                    quote! {
                        self::#languages_enum::#lang => self.#lang_fn_ident(&mut out, #(#fn_vars),*)
                    }
                },
            )
            .collect();

        quote! {
            pub fn #fn_ident #fn_generics(&self, #(#var: impl Into<::fluent_static::value::Value<'a>>),*) -> ::fluent_static::Message {
                #(let #var = #var.into();)*
                let mut out = String::new();
                match self.language {
                    #(#lang_selectors),*,
                }.unwrap();

                ::fluent_static::Message::from(out)
            }
        }
    }

    pub fn build(&self) -> Result<TokenStream2, Error> {
        self.validate()?.generate()
    }
}

impl Default for MessageBundleBuilder {
    fn default() -> Self {
        Self::new("Message")
    }
}
