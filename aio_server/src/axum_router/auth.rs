use async_trait::async_trait;
use axum::{
    Router,
    extract::{Query, State},
    http::{HeaderMap, StatusCode, header::SET_COOKIE},
    response::{IntoResponse, Redirect},
    routing::get,
};
use axum_extra::{TypedHeader, headers};
use oauth2::url::Url;
use serde::Deserialize;
use thiserror::Error;
use tracing::debug;

use crate::axum_router::AppState;

const COOKIE_SESSION_KEY: &str = "SESSION";

#[derive(Debug, Error)]
pub enum OAuth2Error {
    #[error("cannot authenticate user")]
    AuthenticationFailure,

    #[error(transparent)]
    Underlying(#[from] anyhow::Error),
}

/// Trait for [`UserAuthenticator`] that support OAuth2 workflows.
#[async_trait]
pub trait Oauth2Authentifier {
    /// Initiates a session in the authenticator and provides an URL (the callback URL) and cookie
    /// to submit an Authentication request to Discord.
    ///
    /// # Errors
    ///
    /// The function may fail if it cannot register the session in the authenticator.
    async fn init_auth(&self) -> Result<(Url, String), OAuth2Error>;

    /// Callback function used in OAuth2 workflow that takes the output of the Identity Provider and
    /// start a session.
    ///
    /// # Errors
    ///
    /// This function will return an error if it fails decoding and applying the workflow before
    /// starting a session.
    async fn login_callback(
        &self,
        cookie: &str,
        provided_csrf_token: &str,
        provided_code: &str,
    ) -> Result<String, OAuth2Error>;

    /// Removes the session of the session store
    async fn logout(&self, cookie: &str) -> Result<(), OAuth2Error>;
}

/// Make an Axum `Router` that handles `OAuth2` authentication through Discord via the following paths:
///
/// - `/auth/discord` initiates the OAuth workflow and redirects to Discord URL.
/// - `/auth/discord/authorized` is the endpoint used as callback once the authentication has been
///   performed.
/// - `/logout` is the endpoint for letting the user logout.
pub fn mk_router() -> Router<AppState> {
    Router::new()
        .route("/auth/discord", get(discord_auth))
        .route("/auth/discord/authorized", get(discord_login_authorized))
        .route("/logout", get(logout))
}

/// Handler for initiating the OAuth workflow.
async fn discord_auth(
    State(AppState {
        discord_authenticator,
    }): State<AppState>,
) -> impl IntoResponse {
    debug!("received an authentication request through Discord, using discord authenticator");
    let Ok((url, cookie)) = discord_authenticator.init_auth().await else {
        return (StatusCode::BAD_REQUEST, "failed to get discord auth url").into_response();
    };

    let cookie = format!("{COOKIE_SESSION_KEY}={cookie}; SameSite=Lax; Path=/");
    let Ok(cookie) = cookie.parse() else {
        return (StatusCode::BAD_REQUEST, "cannot parse cookie").into_response();
    };

    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, cookie);

    (headers, Redirect::to(url.as_ref())).into_response()
}

#[derive(Debug, Deserialize)]
struct AuthRequest {
    code: String,
    state: String,
}

/// Handler for receiveing the redirection once authentication to discord has been performed.
async fn discord_login_authorized(
    Query(AuthRequest { code, state }): Query<AuthRequest>,
    State(AppState {
        discord_authenticator,
    }): State<AppState>,
    TypedHeader(cookies): TypedHeader<headers::Cookie>,
) -> impl IntoResponse {
    let Some(session_cookie) = cookies.get(COOKIE_SESSION_KEY) else {
        return (StatusCode::BAD_REQUEST, "missing SESSION cookie").into_response();
    };

    let Ok(cookie) = discord_authenticator
        .login_callback(session_cookie, &state, &code)
        .await
    else {
        return (
            StatusCode::BAD_REQUEST,
            "cannot create session with the provided parameters",
        )
            .into_response();
    };

    let Ok(cookie) =
        format!("{COOKIE_SESSION_KEY}={cookie}; SameSite=Lax; HttpOnly; Secure; Path=/").parse()
    else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "cannot serialize cookie, just mhe",
        )
            .into_response();
    };

    // Set session cookie
    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, cookie);

    (headers, Redirect::to("/")).into_response()
}

/// Handler for the logout endpoint
async fn logout(
    State(AppState {
        discord_authenticator,
    }): State<AppState>,
    TypedHeader(cookies): TypedHeader<headers::Cookie>,
) -> impl IntoResponse {
    let Some(session_cookie) = cookies.get(COOKIE_SESSION_KEY) else {
        return (StatusCode::BAD_REQUEST, "missing SESSION cookie").into_response();
    };

    if discord_authenticator.logout(session_cookie).await.is_err() {
        return (StatusCode::INTERNAL_SERVER_ERROR, "cannot logout").into_response();
    }

    Redirect::to("/").into_response()
}
