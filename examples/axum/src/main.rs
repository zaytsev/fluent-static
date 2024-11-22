use axum::{routing::get, Router};
use fluent_static::support::axum::{RequestLanguage, RequestLanguageConfig};
use maud::{html, Markup};

mod l10n {
    use fluent_static::message_bundle;
    #[message_bundle(
        resources = [
            ("l10n/en-US/messages.ftl", "en-US"), 
            ("l10n/fr-CH/messages.ftl", "fr-CH"), 
        ],
        default_language = "en-US")]
    pub struct Messages;
}

#[tokio::main]
async fn main() {
    // By default, only the `Accept-Language` HTTP header is used to determine the
    // client's preferred language.
    // The `RequestLanguageConfig` struct allows you to configure the language selection.
    // This includes
    // - `ignore_accept_language_header`,
    // - `language_cookie_name` and
    // - `fallback_language_id`.

    // In the following configuration, the cookie name is set to `"lang"`.
    // If this cookie is present, the value will be used instead of the `Accept-Language` header.
    let request_lang_config = RequestLanguageConfig::builder()
        .language_cookie_name("lang")
        .build();

    let app = Router::new()
        .route("/", get(handler))
        // The configuration must then be set as an axum request extension,
        // which will apply it to all routes.
        // See https://docs.rs/axum/latest/axum/#using-request-extensions
        .layer(axum::Extension(request_lang_config));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn handler(RequestLanguage(msgs): RequestLanguage<l10n::Messages>) -> Markup {
    let name = "Guest";
    html! {
        html {
            head {
                title {
                    (msgs.page_title())
                }
            }
            body {
                h1 {
                    (msgs.hello(name))
                }
            }
        }
    }
}
