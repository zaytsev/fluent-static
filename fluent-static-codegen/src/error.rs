use std::path::PathBuf;

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
        mismatching_messages: Vec<(String, String, Vec<String>)>,
    },
}
