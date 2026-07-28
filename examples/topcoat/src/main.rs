use fluent_static::support::topcoat::{request_language, RequestLanguageConfig};
use topcoat::{
    cookie::RouterBuilderCookieExt,
    context::Cx,
    router::{layout, page, Router},
    view::view,
    Result,
};

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

#[layout("/")]
async fn root_layout(slot: Result) -> Result {
    view! {
        <!DOCTYPE html>
        (slot?)
    }
}

#[page("/")]
async fn handler(cx: &Cx) -> Result {
    let msgs = request_language::<l10n::Messages>(cx);
    let name = "Guest";
    view! {
        <html>
            <head>
                <title>(msgs.page_title())</title>
            </head>
            <body>
                <h1>(msgs.hello(name))</h1>
            </body>
        </html>
    }
}

#[tokio::main]
async fn main() {
    // By default, only the `Accept-Language` HTTP header is used to determine the
    // client's preferred language.
    // The `RequestLanguageConfig` struct allows you to configure the language selection.
    // This includes:
    // - `ignore_accept_language_header`,
    // - `language_cookie_name` and
    // - `fallback_language_id`.

    // In the following configuration, the cookie name is set to `"lang"`.
    // If this cookie is present, the value will be used instead of the `Accept-Language` header.
    let request_lang_config = RequestLanguageConfig::builder()
        .language_cookie_name("lang")
        .build();

    // Register the config as app context (keyed by type) and install cookie support.
    let router = Router::builder()
        .app_context(request_lang_config)
        .cookies()
        .layout(root_layout)
        .page(handler)
        .build();

    // Start the server on the default host and port (127.0.0.1:3000).
    topcoat::start(router).await.unwrap();
}
