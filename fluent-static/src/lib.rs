use std::{borrow::Cow, fmt::Display};

pub use accept_language;
pub use once_cell;
pub use unic_langid;

pub mod fluent_bundle {
    pub use fluent_bundle::concurrent::FluentBundle;
    pub use fluent_bundle::{FluentArgs, FluentError, FluentMessage, FluentResource, FluentValue};
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Message<'a>(Cow<'a, str>);

impl<'a> Display for Message<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(feature = "maud")]
impl<'a> maud::Render for Message<'a> {
    fn render_to(&self, buffer: &mut String) {
        str::render_to(&self.0, buffer);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AcceptedLanguage(String);

impl<'a> AsRef<str> for AcceptedLanguage {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(feature = "axum")]
pub mod axum {
    use async_trait::async_trait;
    use axum_core::extract::FromRequestParts;
    use http::{header::ACCEPT_LANGUAGE, request::Parts, StatusCode};

    use crate::AcceptedLanguage;

    #[async_trait]
    impl<S> FromRequestParts<S> for super::AcceptedLanguage
    where
        S: Send + Sync,
    {
        type Rejection = (StatusCode, &'static str);

        async fn from_request_parts(
            parts: &mut Parts,
            _state: &S,
        ) -> Result<Self, Self::Rejection> {
            let accept_language = parts
                .headers
                .get(ACCEPT_LANGUAGE)
                .ok_or((StatusCode::BAD_REQUEST, "Accept-Language header is missing"))?
                .to_str()
                .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid Accept-Language header"))?;

            Ok(AcceptedLanguage(accept_language.into()))
        }
    }
}
