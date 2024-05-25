use axum::{routing::get, Router};
use fluent_static::axum::RequestLanguage;
use fluent_static::LanguageSpec;
use maud::{html, Markup};

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(handler));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn handler(RequestLanguage(lang): RequestLanguage<LanguageSpec>) -> Markup {
    let name = "Guest";
    html! {
        html {
            head {
                title {
                    (l10n::messages::page_title(&lang).unwrap())
                }
            }
            body {
                h1 {
                    (l10n::messages::hello(&lang, name).unwrap())
                }
            }
        }
    }
}
mod l10n {
    include!(concat!(env!("OUT_DIR"), "/l10n.rs"));
}
