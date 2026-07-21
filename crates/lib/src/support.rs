#[cfg(feature = "maud")]
pub mod maud {
    use crate::Message;

    impl maud::Render for Message {
        fn render_to(&self, buffer: &mut String) {
            str::render_to(&self.0, buffer);
        }
    }
}

#[cfg(any(feature = "axum", feature = "topcoat"))]
mod http {
    use std::sync::Arc;

    #[derive(Debug, Default)]
    pub struct RequestLanguageConfigInner {
        pub skip_language_header: bool,
        pub language_cookie: Option<String>,
        pub default_language: Option<String>,
    }

    #[derive(Debug, Clone, Default)]
    pub struct RequestLanguageConfig {
        pub inner: Arc<RequestLanguageConfigInner>,
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
        pub inner: RequestLanguageConfigInner,
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
}

#[cfg(feature = "axum")]
pub mod axum {
    use std::future::Future;

    use accept_language;
    use axum_core::extract::FromRequestParts;
    use axum_extra::extract::CookieJar;
    use http::{header::ACCEPT_LANGUAGE, request::Parts, StatusCode};

    use crate::MessageBundle;

    pub use super::http::{RequestLanguageConfig, RequestLanguageConfigBuilder};

    pub struct RequestLanguage<T: MessageBundle>(pub T);

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

#[cfg(feature = "topcoat")]
pub mod topcoat {
    use crate::{Message, MessageBundle};

    use accept_language;
    use http::header::ACCEPT_LANGUAGE;
    use topcoat::{
        context::{try_app_context, Cx},
        cookie::{cookies, Cookies},
        router::headers,
        view::{NodeViewParts, PartsWriter},
    };

    pub use super::http::{RequestLanguageConfig, RequestLanguageConfigBuilder};

    impl NodeViewParts for Message {
        fn into_view_parts(self, _: &Cx, parts: &mut PartsWriter<'_>) {
            parts.push_str(self.0);
        }
    }

    pub fn request_language<T>(cx: &Cx) -> T
    where
        T: MessageBundle,
    {
        let cfg = try_app_context::<RequestLanguageConfig>(cx)
            .cloned()
            .unwrap_or_default();

        if let Some(cookie_name) = cfg.language_cookie_name() {
            if let Some(bundle) = cookies(cx)
                .get(cookie_name)
                .map(|cookie| cookie.value().to_owned())
                .and_then(|val| T::get(&val))
            {
                return bundle;
            }
        }

        let bundle = if !cfg.ignore_language_header() {
            headers(cx)
                .get(ACCEPT_LANGUAGE)
                .and_then(|v| v.to_str().ok())
                .and_then(|value| {
                    accept_language::intersection_ordered(value, T::supported_language_ids())
                        .first()
                        .and_then(|lang| T::get(lang))
                })
        } else {
            None
        };

        bundle
            .or_else(|| cfg.fallback_language_id().and_then(T::get))
            .unwrap_or_default()
    }

    #[cfg(test)]
    mod tests {
        use std::sync::Arc;

        use crate::LanguageAware;

        use super::*;
        use http::{header::ACCEPT_LANGUAGE, Request};
        use topcoat::{
            context::{ContextMap, CxBuilder},
            cookie::CookieJarCell,
        };

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

        fn build_cx(
            config: RequestLanguageConfig,
            parts: http::request::Parts,
            with_cookies: bool,
        ) -> Cx {
            let mut app_context = ContextMap::new();
            app_context.insert(config);

            let mut cx_builder = CxBuilder::new(Arc::new(app_context));
            cx_builder.insert(parts);
            if with_cookies {
                cx_builder.insert(CookieJarCell::new());
            }
            cx_builder.build()
        }

        #[test]
        fn test_language_from_header() {
            let req = Request::builder()
                .header(ACCEPT_LANGUAGE, "en-US,en;q=0.5")
                .body(String::default())
                .unwrap();

            let cx = build_cx(RequestLanguageConfig::default(), req.into_parts().0, false);

            assert_eq!(
                request_language::<LanguageSpec>(&cx),
                LanguageSpec("en".to_string())
            );
        }

        #[test]
        fn test_language_from_cookie() {
            let cookie_name = "lang";
            let cookie_value = "de";

            let req = Request::builder()
                .header("Cookie", format!("{}={}", cookie_name, cookie_value))
                .body(String::default())
                .unwrap();

            let cx = build_cx(
                RequestLanguageConfig::builder()
                    .ignore_accept_language_header(true)
                    .language_cookie_name(cookie_name)
                    .build(),
                req.into_parts().0,
                true,
            );

            assert_eq!(
                request_language::<LanguageSpec>(&cx),
                LanguageSpec("de".to_string())
            );
        }

        #[test]
        fn test_fallback_language() {
            let req = Request::builder().body(String::default()).unwrap();

            let cx = build_cx(
                RequestLanguageConfig::builder()
                    .fallback_language_id("fr")
                    .build(),
                req.into_parts().0,
                false,
            );

            assert_eq!(
                request_language::<LanguageSpec>(&cx),
                LanguageSpec("fr".to_string())
            );
        }

        #[test]
        fn test_no_language_specified() {
            let req = Request::builder().body(String::default()).unwrap();

            let cx = build_cx(RequestLanguageConfig::default(), req.into_parts().0, false);

            assert_eq!(
                request_language::<LanguageSpec>(&cx),
                LanguageSpec::default()
            );
        }
    }
}
