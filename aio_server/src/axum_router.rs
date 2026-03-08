use async_session::MemoryStore;
use axum::{Router, extract::MatchedPath, http::Request, response::Html, routing::get};
use tower_http::trace::TraceLayer;
use tracing::trace_span;

mod auth;
pub mod discord_authenticator;
use discord_authenticator::DiscordAuthenticator;

async fn test_handler() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}

#[derive(Clone)]
struct AppState {
    discord_authenticator: DiscordAuthenticator<MemoryStore>,
}

pub fn mk_router(discord_authenticator: DiscordAuthenticator<MemoryStore>) -> Router {
    let app_state = AppState {
        discord_authenticator,
    };

    Router::new()
        .route("/", get(test_handler))
        .merge(auth::mk_router())
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
                // Log the matched route's path (with placeholders not filled in).
                // Use request.uri() or OriginalUri if you want the real path.
                let path = request
                    .extensions()
                    .get::<MatchedPath>()
                    .map(MatchedPath::as_str);

                trace_span!(
                    "http_request",
                    method = ?request.method(),
                    path,
                )
            }),
        )
        .with_state(app_state)
}
