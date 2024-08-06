use std::{collections::BTreeSet, path::PathBuf};

use crate::message::NormalizedMessage;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    PathPrefixError(#[from] std::path::StripPrefixError),

    #[error("Path contans invalid symbols: {0}")]
    InvalidPath(PathBuf),

    #[error("Path doesn not match the expected format {0}")]
    InvalidPathFormat(PathBuf),

    #[error("Error parsing fluent resource")]
    FluentParserError {
        errors: Vec<fluent_syntax::parser::ParserError>,
    },

    #[error("Found unsupported feature {feature}: {id}")]
    UnsupportedFeature { feature: String, id: String },

    #[error("No l10n resources found for fallback language {0}")]
    FallbackLanguageNotFound(String),

    #[error("Message bundle {bundle} integrity validation failed")]
    MessageBundleValidationError {
        bundle: String,
        path: String,
        entries: Vec<MessageValidationErrorEntry>,
    },
}

#[derive(Debug)]
pub struct MessageValidationErrorEntry {
    pub message: NormalizedMessage,
    pub defined_in_languages: BTreeSet<String>,
    pub undefined_in_languages: BTreeSet<String>,
}
