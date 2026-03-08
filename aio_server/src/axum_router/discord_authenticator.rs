use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, anyhow};
use async_session::{MemoryStore, Session, SessionStore};
use async_trait::async_trait;
use axum::extract::{FromRef, FromRequestParts, OptionalFromRequestParts};
use axum::http::request::Parts;
use oauth2::basic::{BasicClient, BasicErrorResponseType, BasicTokenType};
use oauth2::url::Url;
use oauth2::{
    AuthUrl, AuthorizationCode, Client, ClientId, ClientSecret, CsrfToken, EmptyExtraTokenFields,
    EndpointNotSet, EndpointSet, RedirectUrl, RevocationErrorResponseType, Scope,
    StandardErrorResponse, StandardRevocableToken, StandardTokenIntrospectionResponse,
    StandardTokenResponse, TokenResponse, TokenUrl,
};
use serde::Deserialize;
use tracing::instrument;

use crate::axum_router::AppState;
use crate::axum_router::auth::{OAuth2Error, Oauth2Authentifier};

const DISCORD_AUTH_URI: &str = "https://discord.com/api/oauth2/authorize?response_type=code";
const DISCORD_TOKEN_URI: &str = "https://discord.com/api/oauth2/token";
const DISCORD_USER_URI: &str = "https://discordapp.com/api/users/@me";

const SCOPE_EMAIL: &str = "email";
const SCOPE_IDENTIFY: &str = "identify";
const SCOPE_GUILDS: &str = "guilds";

const SESSION_STORE_CSRF_TOKEN_KEY: &str = "csrf_token";
const SESSION_STORE_USER_KEY: &str = "user";
const SESSION_DURATION_SECS: u64 = 3600;

type DiscordOAuthClient = Client<
    StandardErrorResponse<BasicErrorResponseType>,
    StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>,
    StandardTokenIntrospectionResponse<EmptyExtraTokenFields, BasicTokenType>,
    StandardRevocableToken,
    StandardErrorResponse<RevocationErrorResponseType>,
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointSet,
>;

/// `DiscordAuthenticator`
#[derive(Debug, Clone)]
pub struct DiscordAuthenticator<S>
where
    S: SessionStore,
{
    oauth_client: DiscordOAuthClient,
    session_store: Arc<S>,
    reqwest_client: oauth2::reqwest::Client,
}

/// Structure used to deserialize Discord API responses to the GET /api/users/@me endpoint.
/// Extensive list of fields returned by the endpoint can be found
/// [here](https://discord.com/developers/docs/resources/user#user-object)
#[allow(dead_code)]
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize)]
struct DiscordUser {
    id: String,
    avatar: Option<String>,
    username: String,
    discriminator: String,
    email: String,
}

impl<S> DiscordAuthenticator<S>
where
    S: SessionStore,
{
    /// Instanciates a new discord authenticator.
    ///
    /// # Panics
    ///
    /// Panics if the internal hardcoded values of Discord URL are not parsable by the `oauth` crate.
    ///
    /// # Errors
    ///
    /// This function will return an error if the `redirect_url` parameter cannot be parsed as a URL.
    pub fn new(
        client_id: &str,
        client_secret: &str,
        redirect_url: &str,
        session_store: Arc<S>,
    ) -> Result<Self, OAuth2Error> {
        let oauth_client = BasicClient::new(ClientId::new(client_id.to_string()))
            .set_client_secret(ClientSecret::new(client_secret.to_string()))
            .set_auth_uri(
                AuthUrl::new(DISCORD_AUTH_URI.to_string())
                    .expect("Failed to create an AuthUrl from discord hard-coded value"),
            )
            .set_token_uri(
                TokenUrl::new(DISCORD_TOKEN_URI.to_string())
                    .expect("Failed to create a TokenUrl from discord hard-coded value"),
            )
            .set_redirect_uri(
                RedirectUrl::new(redirect_url.to_string())
                    .context("failed to parse redirect_url")?,
            );

        let reqwest_client = oauth2::reqwest::ClientBuilder::new()
            .redirect(oauth2::reqwest::redirect::Policy::none())
            .build()
            .context("cannot build reqwest client")?;

        Ok(Self {
            oauth_client,
            session_store,
            reqwest_client,
        })
    }
}

#[async_trait]
impl<S> Oauth2Authentifier for DiscordAuthenticator<S>
where
    S: SessionStore,
{
    #[instrument(skip_all)]
    async fn init_auth(&self) -> Result<(Url, String), OAuth2Error> {
        let oauth_client = self.oauth_client.clone();

        let (auth_url, csrf_token) = oauth_client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new(SCOPE_IDENTIFY.to_string()))
            .add_scope(Scope::new(SCOPE_EMAIL.to_string()))
            .add_scope(Scope::new(SCOPE_GUILDS.to_string()))
            .url();

        let mut session = Session::new();
        session
            .insert(SESSION_STORE_CSRF_TOKEN_KEY, csrf_token.secret())
            .context("cannot insert the csrf_token in the session")?;
        session.expire_in(Duration::from_secs(SESSION_DURATION_SECS));

        let Ok(Some(cookie)) = self.session_store.store_session(session).await else {
            return Err(OAuth2Error::Underlying(anyhow!("cannot store session")));
        };

        Ok((auth_url, cookie))
    }

    #[instrument(skip_all)]
    async fn login_callback(
        &self,
        cookie: &str,
        provided_csrf_token: &str,
        provided_code: &str,
    ) -> Result<String, OAuth2Error> {
        // In a first time validate the CSRF workflow with the CSRF token stored in the SessionStore
        let Ok(Some(session)) = self.session_store.load_session(cookie.to_string()).await else {
            return Err(OAuth2Error::Underlying(anyhow!(
                "csrf entry in session store not found"
            )));
        };

        let Some(stored_csrf_token) = session.get::<CsrfToken>(SESSION_STORE_CSRF_TOKEN_KEY) else {
            return Err(OAuth2Error::AuthenticationFailure);
        };
        if provided_csrf_token != stored_csrf_token.secret() {
            return Err(OAuth2Error::AuthenticationFailure);
        }

        // When validated the CSRF Token can be removed from the store.
        self.session_store
            .destroy_session(session)
            .await
            .context("failed to destroy the session in the session store")?;

        // Use the token to query Discord API and fetch User information.
        let token = self
            .oauth_client
            .exchange_code(AuthorizationCode::new(provided_code.to_string()))
            .request_async(&self.reqwest_client)
            .await
            .context("failed to exchange authorization code")?;

        let user_info = self
            .reqwest_client
            .get(DISCORD_USER_URI)
            .bearer_auth(token.access_token().secret())
            .send()
            .await
            .context("request to get discord's user information failed")?
            .json::<DiscordUser>()
            .await
            .context("deserialization of Discord User informations failed")?;

        // Register the user session in the session store
        let mut session = Session::new();
        session
            .insert(SESSION_STORE_USER_KEY, &user_info.email)
            .context("failed in inserting serialized value into session")?;

        let Ok(Some(cookie)) = self.session_store.store_session(session).await else {
            return Err(OAuth2Error::Underlying(anyhow!(
                "user entry in session store not found"
            )));
        };

        Ok(cookie)
    }

    #[instrument(skip_all)]
    async fn logout(&self, cookie: &str) -> Result<(), OAuth2Error> {
        let Ok(Some(session)) = self.session_store.load_session(cookie.to_string()).await else {
            return Err(OAuth2Error::Underlying(anyhow!("session entry not found")));
        };

        self.session_store
            .destroy_session(session)
            .await
            .map_err(OAuth2Error::Underlying)
    }
}

impl<S> AsRef<DiscordOAuthClient> for DiscordAuthenticator<S>
where
    S: SessionStore,
{
    fn as_ref(&self) -> &DiscordOAuthClient {
        &self.oauth_client
    }
}

impl<S> FromRef<AppState> for DiscordAuthenticator<S>
where
    S: SessionStore,
{
    fn from_ref(state: &AppState) -> Self {
        state.discord_authenticator.clone()
    }
}

impl<S> FromRequestParts<S> for Identity
where
    MemoryStore: FromRef<S>,
    S: Send + Sync,
{
    // If anything goes wrong or no session is found, redirect to the auth page
    type Rejection = AuthRedirect;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        todo!();
    }
}

impl<S> OptionalFromRequestParts<S> for Identity
where
    MemoryStore: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Option<Self>, Self::Rejection> {
        todo!();
    }
}
