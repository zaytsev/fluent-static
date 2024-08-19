use std::{collections::BTreeSet, path::PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error("Error reading resource {path}")]
    ResourceReadError {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("Error reading resource from '{0}' while no `base_dir` is configured")]
    UnexpectedRelativePath(PathBuf),

    #[error(transparent)]
    PathPrefixError(#[from] std::path::StripPrefixError),

    #[error("Path contans invalid symbols: {0}")]
    InvalidPathSymbol(PathBuf),

    #[error("Path doesn not match the expected format {0}")]
    InvalidPathFormat(PathBuf),

    #[error("Error parsing fluent resource")]
    FluentParserError {
        errors: Vec<fluent_syntax::parser::ParserError>,
    },

    #[error(transparent)]
    InvalidLanguageId(#[from] unic_langid::LanguageIdentifierError),

    #[error("Error parsing Fluent resource {path}")]
    FluentResourceParseError {
        path: PathBuf,
        errors: Vec<fluent_syntax::parser::ParserError>,
    },

    #[error(transparent)]
    SyntaxError(#[from] syn::Error),

    #[error("Found unsupported feature {feature}: {id}")]
    UnsupportedFeature { feature: String, id: String },

    #[error("No l10n resources found for fallback language {0}")]
    FallbackLanguageNotFound(String),

    #[error("Message bundle {bundle} integrity validation failed")]
    MessageBundleValidationError {
        bundle: String,
        path: Option<String>,
        entries: Vec<MessageValidationErrorEntry>,
    },

    #[error("Message bundle builder context is in an invalid state")]
    UnexpectedContextState,

    #[error("Error parsing literal value {0}")]
    InvalidLiteral(String),

    #[error(transparent)]
    LexErr(#[from] proc_macro2::LexError),

    #[error("Found duplicated entry with ID '{0}'")]
    DuplicateEntryId(String),

    #[error("Message bundle default language '{lang}' has not corresponding fluent resources")]
    UnsupportedDefaultLanguage { lang: String },

    #[error("Message {message_id} selector must have exactly one default variant")]
    InvalidSelectorDefaultVariant { message_id: String },
}

#[derive(Debug)]
pub struct MessageValidationErrorEntry {
    pub message_id: String,
    pub defined_in_languages: BTreeSet<String>,
    pub undefined_in_languages: BTreeSet<String>,
}
