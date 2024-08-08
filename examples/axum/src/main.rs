use axum::{routing::get, Router};
use fluent_static::axum::RequestLanguage;
use maud::{html, Markup};

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(handler));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn handler(RequestLanguage(msgs): RequestLanguage<l10n::messages::MessagesBundle>) -> Markup {
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
                    (msgs.hello(name).unwrap())
                }
            }
        }
    }
}
mod l10n {
    fluent_static::include_source!("l10n.rs");
}
