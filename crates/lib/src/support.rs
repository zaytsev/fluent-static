#[cfg(feature = "maud")]
pub mod maud {
    use crate::Message;

    impl maud::Render for Message {
        fn render_to(&self, buffer: &mut String) {
            str::render_to(&self.0, buffer);
        }
    }
}

#[cfg(feature = "axum")]
pub mod axum {
    use std::{future::Future, sync::Arc};

    use accept_language;
    use axum_core::extract::FromRequestParts;
    use axum_extra::extract::CookieJar;
    use http::{header::ACCEPT_LANGUAGE, request::Parts, StatusCode};

    use crate::MessageBundle;

    pub struct RequestLanguage<T: MessageBundle>(pub T);

    #[derive(Debug, Default)]
    struct RequestLanguageConfigInner {
        skip_language_header: bool,
        language_cookie: Option<String>,
        default_language: Option<String>,
    }

    #[derive(Debug, Clone, Default)]
    pub struct RequestLanguageConfig {
        inner: Arc<RequestLanguageConfigInner>,
    }

    impl RequestLanguageConfig {
        pub fn builder() -> RequestLanguageConfigBuilder {
            RequestLanguageConfigBuilder {
                inner: RequestLanguageConfigInner::default(),
            }
        }

        pub fn ignore_language_header(&self) -> bool {
            self.inner.skip_language_header
        }

        pub fn language_cookie_name(&self) -> Option<&str> {
            self.inner.language_cookie.as_deref()
        }

        pub fn fallback_language_id(&self) -> Option<&str> {
            self.inner.default_language.as_deref()
        }
    }

    pub struct RequestLanguageConfigBuilder {
        inner: RequestLanguageConfigInner,
    }

    impl RequestLanguageConfigBuilder {
        pub fn ignore_accept_language_header(mut self, value: bool) -> Self {
            self.inner.skip_language_header = value;
            self
        }

        pub fn language_cookie_name(mut self, value: &str) -> Self {
            self.inner.language_cookie = Some(value.to_string());
            self
        }

        pub fn fallback_language_id(mut self, value: &str) -> Self {
            self.inner.default_language = Some(value.to_string());
            self
        }

        pub fn build(self) -> RequestLanguageConfig {
            RequestLanguageConfig {
                inner: Arc::new(self.inner),
            }
        }
    }

    impl<T: MessageBundle, S> FromRequestParts<S> for RequestLanguage<T>
    where
        S: Send + Sync,
    {
        type Rejection = (StatusCode, &'static str);

        fn from_request_parts(
            parts: &mut Parts,
            _state: &S,
        ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
            async move {
                let cfg = parts
                    .extensions
                    .get::<RequestLanguageConfig>()
                    .map(|cfg| cfg.clone())
                    .unwrap_or_default();

                if let Some(cookie_name) = cfg.language_cookie_name().as_ref() {
                    if let Some(bundle) = CookieJar::from_request_parts(parts, _state)
                        .await
                        .ok()
                        .and_then(|jar| {
                            jar.get(cookie_name)
                                .map(|cookie| cookie.value_trimmed())
                                .and_then(T::get)
                        })
                    {
                        return Ok(Self(bundle));
                    };
                };

                let bundle = if !cfg.ignore_language_header() {
                    parts
                        .headers
                        .get(ACCEPT_LANGUAGE)
                        .and_then(|v| v.to_str().ok())
                        .and_then(|value| {
                            accept_language::intersection_ordered(
                                value,
                                T::supported_language_ids(),
                            )
                            .first()
                            .and_then(|lang| T::get(lang))
                        })
                } else {
                    None
                };

                Ok(Self(
                    bundle
                        .or_else(|| cfg.fallback_language_id().and_then(T::get))
                        .unwrap_or_default(),
                ))
            }
        }
    }

    #[cfg(test)]
    mod tests {

        use crate::LanguageAware;

        use super::*;
        use http::Request;

        #[derive(Debug, Clone, PartialEq, Eq)]
        struct LanguageSpec(String);

        impl Default for LanguageSpec {
            fn default() -> Self {
                Self::get("en").unwrap()
            }
        }

        impl LanguageAware for LanguageSpec {
            fn language_id(&self) -> &str {
                &self.0
            }
        }

        impl MessageBundle for LanguageSpec {
            fn get(language_id: &str) -> Option<Self>
            where
                Self: Sized,
            {
                match language_id {
                    "en" => Some(LanguageSpec("en".to_string())),
                    "de" => Some(LanguageSpec("de".to_string())),
                    "fr" => Some(LanguageSpec("fr".to_string())),
                    _ => None,
                }
            }

            fn default_language_id() -> &'static str {
                "en"
            }

            fn supported_language_ids() -> &'static [&'static str] {
                &["de", "en", "fr"]
            }
        }

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
                LanguageSpec("en".to_string())
            );
        }

        #[tokio::test]
        async fn test_language_from_cookie() {
            let cookie_name = "lang";
            let cookie_value = "de";

            // Create a fake request with the specific cookie set
            let mut req = Request::builder()
                .header("Cookie", format!("{}={}", cookie_name, cookie_value))
                .header(ACCEPT_LANGUAGE, "en-US,en;q=0.5")
                .body(String::default())
                .unwrap();
            req.extensions_mut().insert(
                RequestLanguageConfig::builder()
                    .ignore_accept_language_header(true)
                    .language_cookie_name(cookie_name)
                    .build(),
            );

            let parts = &mut req.into_parts().0;
            assert_eq!(
                RequestLanguage::<LanguageSpec>::from_request_parts(parts, &())
                    .await
                    .unwrap()
                    .0,
                LanguageSpec("de".to_string())
            );
        }

        #[tokio::test]
        async fn test_default_language() {
            let mut req = Request::builder().body(String::default()).unwrap();

            req.extensions_mut().insert(
                RequestLanguageConfig::builder()
                    .fallback_language_id("fr")
                    .build(),
            );

            let parts = &mut req.into_parts().0;

            assert_eq!(
                RequestLanguage::<LanguageSpec>::from_request_parts(parts, &())
                    .await
                    .unwrap()
                    .0,
                LanguageSpec("fr".to_string())
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
