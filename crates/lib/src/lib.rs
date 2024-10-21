pub use accept_language;
pub use once_cell;
pub use unic_langid;

pub use intl_pluralrules;

pub use fluent_static_function as function;
pub use fluent_static_macros::message_bundle;
pub use fluent_static_value as value;

mod message;
pub use message::Message;
pub mod formatter;

#[macro_export]
macro_rules! include_source {
    ($name:expr) => {
        include!(concat!(env!("OUT_DIR"), "/generated/fluent/", $name));
    };
}

pub trait LanguageAware {
    fn language_id(&self) -> &str;
}

pub trait MessageBundle: LanguageAware {
    fn get(language_id: &str) -> Option<Self>
    where
        Self: Sized;
    fn default_language_id() -> &'static str;
    fn supported_language_ids() -> &'static [&'static str];
}

#[cfg(feature = "maud")]
impl maud::Render for Message {
    fn render_to(&self, buffer: &mut String) {
        str::render_to(&self.0, buffer);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct LanguageSpec(String);

impl LanguageSpec {
    pub fn new(value: String) -> Self {
        Self(value)
    }
}

impl AsRef<str> for LanguageSpec {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl LanguageAware for LanguageSpec {
    fn language_id(&self) -> &str {
        &self.0
    }
}

#[cfg(feature = "axum")]
pub mod axum {
    use async_trait::async_trait;
    use axum_core::extract::FromRequestParts;
    use axum_extra::extract::CookieJar;
    use http::{header::ACCEPT_LANGUAGE, request::Parts, StatusCode};

    use crate::LanguageSpec;

    pub struct RequestLanguage<T: From<LanguageSpec>>(pub T);

    #[derive(Debug, Clone, Default)]
    pub struct RequestLanguageConfig {
        pub skip_language_header: bool,
        pub language_cookie: Option<String>,
        pub default_language: Option<String>,
    }

    #[async_trait]
    impl<T: From<LanguageSpec>, S> FromRequestParts<S> for RequestLanguage<T>
    where
        S: Send + Sync,
    {
        type Rejection = (StatusCode, &'static str);

        async fn from_request_parts(
            parts: &mut Parts,
            _state: &S,
        ) -> Result<Self, Self::Rejection> {
            let cfg = parts
                .extensions
                .get::<RequestLanguageConfig>()
                .map(|cfg| cfg.clone())
                .unwrap_or_default();

            let cookie_spec = if let Some(cookie_name) = cfg.language_cookie.as_ref() {
                if let Ok(jar) = CookieJar::from_request_parts(parts, _state).await {
                    jar.get(cookie_name)
                        .map(|cookie| LanguageSpec(cookie.value_trimmed().to_string()))
                } else {
                    None
                }
            } else {
                None
            };

            let header = parts
                .headers
                .get(ACCEPT_LANGUAGE)
                .and_then(|v| v.to_str().ok())
                .map(|s| LanguageSpec::new(s.to_string()));

            let default_spec = cfg
                .default_language
                .as_ref()
                .map(|v| LanguageSpec(v.to_string()));

            Ok(Self(
                cookie_spec
                    .or(header)
                    .or(default_spec)
                    .unwrap_or_default()
                    .into(),
            ))
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use http::Request;

        #[tokio::test]
        async fn test_language_from_header() {
            let req = Request::builder()
                .header(ACCEPT_LANGUAGE, "en-US,en;q=0.5")
                .body(String::default())
                .unwrap();

            let parts = &mut req.into_parts().0;
            parts.extensions.insert(RequestLanguageConfig::default());

            assert_eq!(
                RequestLanguage::<LanguageSpec>::from_request_parts(parts, &())
                    .await
                    .unwrap()
                    .0,
                LanguageSpec::new("en-US,en;q=0.5".to_string())
            );
        }

        #[tokio::test]
        async fn test_language_from_cookie() {
            let cookie_name = "lang";
            let cookie_value = "de-DE";

            // Create a fake request with the specific cookie set
            let mut req = Request::builder()
                .header("Cookie", format!("{}={}", cookie_name, cookie_value))
                .header(ACCEPT_LANGUAGE, "en-US,en;q=0.5")
                .body(String::default())
                .unwrap();
            req.extensions_mut().insert(RequestLanguageConfig {
                skip_language_header: true,
                language_cookie: Some(cookie_name.to_string()),
                default_language: None,
            });

            let parts = &mut req.into_parts().0;
            assert_eq!(
                RequestLanguage::<LanguageSpec>::from_request_parts(parts, &())
                    .await
                    .unwrap()
                    .0,
                LanguageSpec::new("de-DE".to_string())
            );
        }

        #[tokio::test]
        async fn test_default_language() {
            let mut req = Request::builder().body(String::default()).unwrap();

            req.extensions_mut().insert(RequestLanguageConfig {
                skip_language_header: true,
                language_cookie: None,
                default_language: Some("fr-FR".to_string()),
            });

            let parts = &mut req.into_parts().0;

            assert_eq!(
                RequestLanguage::<LanguageSpec>::from_request_parts(parts, &())
                    .await
                    .unwrap()
                    .0,
                LanguageSpec::new("fr-FR".to_string())
            );
        }

        #[tokio::test]
        async fn test_no_language_specified() {
            let mut req = Request::builder().body(String::default()).unwrap();

            req.extensions_mut()
                .insert(RequestLanguageConfig::default());

            let parts = &mut req.into_parts().0;

            assert_eq!(
                RequestLanguage::<LanguageSpec>::from_request_parts(parts, &())
                    .await
                    .unwrap()
                    .0,
                LanguageSpec::default()
            );
        }
    }
}
