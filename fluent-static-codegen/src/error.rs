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
