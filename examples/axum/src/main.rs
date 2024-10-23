use axum::{routing::get, Router};
use fluent_static::support::axum::RequestLanguage;
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
    let app = Router::new().route("/", get(handler));
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
