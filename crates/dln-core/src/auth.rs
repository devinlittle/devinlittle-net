use anyhow::{Context, Result};
use arc_swap::ArcSwap;
use aws_lc_rs::agreement::PrivateKey;
use backend_common::{
    auth::{LoginInput, LoginOutput},
    Claims, UserRoles,
};
use base64::{prelude::BASE64_STANDARD, Engine};
use reqwest::{
    header::{HeaderValue, AUTHORIZATION},
    Client, StatusCode,
};
use std::{
    pin::Pin,
    sync::{Arc, LazyLock},
};
use thiserror::Error;
use uuid::Uuid;

use crate::{
    error::CoreError,
    fs::{save_cookies, save_secrets, COOKIE_STORE, GLOBAL_CONFIG, GLOBAL_SECRETS},
    get_headers, no_auth_client,
};

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("The user hasnt logged in before")]
    Unauthenticated,
    #[error("User doesn't exist or provided incorrect password")]
    Unauthorized,
    #[error("Account is soft banned")]
    AccountLocked,

    #[error("Server failed with request")]
    InternalServerError,

    #[error("Failed to Send Request")]
    RequestFailure,

    #[error("Error with the AUTHED_CLIENT")]
    ClientError,
    //  #[error("Error making a network request")]
    //  Reqwest(reqwest::Error),
}

#[derive(Clone, Debug)]
pub struct Auth {
    pub id: Uuid,
    pub session_id: Uuid,
    pub username: String,
    pub roles: UserRoles,
    pub public_key: Option<Vec<u8>>,
    pub private_key: Option<Arc<PrivateKey>>,
}

pub static AUTH_STATE: LazyLock<AuthState> = LazyLock::new(AuthState::default);

pub struct AuthState {
    inner: ArcSwap<Option<Arc<Auth>>>,
}

impl Default for AuthState {
    fn default() -> Self {
        Self {
            inner: ArcSwap::from_pointee(None),
        }
    }
}

impl AuthState {
    pub fn get(&self) -> Option<Arc<Auth>> {
        let current = self.inner.load();

        match &**current {
            Some(auth) => Some(Arc::clone(auth)),
            None => None,
        }
    }

    pub fn set(&self, auth: Auth) {
        self.inner.store(Arc::new(Some(Arc::new(auth))));
    }

    pub fn clear(&self) {
        self.inner.store(Arc::new(None));
    }

    pub fn is_authenticated(&self) -> bool {
        self.get().is_some()
    }
}

pub static AUTHED_CLIENT: LazyLock<AuthClient> = LazyLock::new(AuthClient::default);

pub struct AuthClient {
    inner: ArcSwap<Option<Client>>,
}

impl Default for AuthClient {
    fn default() -> Self {
        Self {
            inner: ArcSwap::from_pointee(None),
        }
    }
}

impl AuthClient {
    pub fn set(&self, client: Client) {
        self.inner.store(Arc::new(Some(client)));
    }

    pub fn clear(&self) {
        self.inner.store(Arc::new(None));
    }

    fn build_client(token: &str) -> Result<Client, CoreError> {
        let mut headers = get_headers();
        let auth_value = HeaderValue::from_str(&format!("Bearer {}", token))
            .map_err(|_| CoreError::Auth(AuthError::ClientError))?;

        headers.insert(AUTHORIZATION, auth_value);

        Client::builder()
            .default_headers(headers)
            .cookie_provider(Arc::clone(&COOKIE_STORE))
            .build()
            .map_err(|_| CoreError::Auth(AuthError::ClientError))
    }

    pub async fn initialize(&self, token: String) -> Result<(), CoreError> {
        let client = Self::build_client(&token)?;
        self.set(client);
        Ok(())
    }

    fn get_client() -> Result<Client, CoreError> {
        AUTHED_CLIENT
            .inner
            .load()
            .as_ref()
            .clone()
            .ok_or(CoreError::NotInitalized)
    }

    pub async fn execute<F>(&self, builder: F) -> Result<reqwest::Response, CoreError>
    where
        F: for<'a> Fn(
            &'a Client,
        ) -> Pin<
            Box<dyn Future<Output = Result<reqwest::Response, CoreError>> + Send + 'a>,
        >,
    {
        let client = Self::get_client()?;

        let resp = builder(&client)
            .await
            .map_err(|_| CoreError::Auth(AuthError::ClientError))?;

        if resp.status() != reqwest::StatusCode::UNAUTHORIZED {
            return Ok(resp);
        }

        let _ = self.perform_refresh().await;

        let client = Self::get_client()?;

        builder(&client)
            .await
            .map_err(|_| CoreError::Auth(AuthError::ClientError))
    }

    async fn perform_refresh(&self) -> Result<(), CoreError> {
        if refresh().await.is_ok() {
            self.initialize(GLOBAL_SECRETS.load().access_token.to_string())
                .await?;
            Ok(())
        } else {
            self.clear();
            AUTH_STATE.clear();
            logout().await?;
            Err(CoreError::Auth(AuthError::RequestFailure))
        }
    }
}

#[allow(clippy::collapsible_if)]
pub async fn get_ready_for_devin_grfd(called_from_login_page: bool) -> Result<(), CoreError> {
    if !called_from_login_page {
        if let Err(e) = refresh().await {
            // connect_notifications
            println!("ERROR REFRESHING, NOW GUEST USER ON NOTIFICATIONS");

            return Err(e);
        }
    }

    // mount db and notify client if they need to do encryption stuff

    // connect notifications

    Ok(())
}

// TODO: have the openapi doc used instead of hardcoded urls

/// just does a token refresh on inital app load
pub async fn refresh() -> Result<(), CoreError> {
    let secrets = GLOBAL_SECRETS.load();

    let api_url = url::Url::parse(&GLOBAL_CONFIG.load().api_url)
        .map_err(|_| CoreError::Auth(AuthError::InternalServerError))?;

    let has_cookies = {
        let jar = COOKIE_STORE
            .lock()
            .map_err(|_| CoreError::Auth(AuthError::InternalServerError))?;
        jar.get_request_values(&api_url).count() > 0
    };

    if secrets.access_token.is_empty() || !has_cookies {
        return Err(CoreError::Auth(AuthError::Unauthenticated));
    }

    let output = no_auth_client()
        .post(format!("{}/auth/refresh", GLOBAL_CONFIG.load().api_url))
        .send()
        .await
        .map_err(|_| CoreError::Auth(AuthError::RequestFailure))?
        .error_for_status()
        .map_err(|e| match e.status() {
            Some(StatusCode::INTERNAL_SERVER_ERROR) => {
                CoreError::Auth(AuthError::InternalServerError)
            }
            Some(StatusCode::UNAUTHORIZED) => CoreError::Auth(AuthError::Unauthorized),
            Some(StatusCode::FORBIDDEN) => CoreError::Auth(AuthError::AccountLocked),
            _ => CoreError::Auth(AuthError::RequestFailure),
        })?
        .json::<LoginOutput>()
        .await
        .map_err(|_| CoreError::Auth(AuthError::RequestFailure))?;

    let _ = save_cookies();

    let _ = set_token(output.access_token).await;

    Ok(())
}

// TODO: have the openapi doc used instead of hardcoded urls
pub async fn login_req(params: LoginInput) -> Result<(), CoreError> {
    let output = no_auth_client()
        .post(format!("{}/auth/login", GLOBAL_CONFIG.load().api_url))
        .json::<LoginInput>(&params)
        .send()
        .await
        .map_err(|_| CoreError::Auth(AuthError::RequestFailure))?
        .error_for_status()
        .map_err(|e| match e.status() {
            Some(StatusCode::INTERNAL_SERVER_ERROR) => {
                CoreError::Auth(AuthError::InternalServerError)
            }
            Some(StatusCode::UNAUTHORIZED) => CoreError::Auth(AuthError::Unauthorized),
            Some(StatusCode::FORBIDDEN) => CoreError::Auth(AuthError::AccountLocked),
            _ => CoreError::Auth(AuthError::RequestFailure),
        })?
        .json::<LoginOutput>()
        .await
        .map_err(|_| CoreError::Auth(AuthError::RequestFailure))?;

    let _ = save_cookies();

    let _ = set_token(output.access_token).await;

    let _ = get_ready_for_devin_grfd(true).await;

    Ok(())
}

pub async fn set_token(access_token: String) -> Result<()> {
    let jwt = jsonwebtoken::dangerous::insecure_decode::<Claims>(&access_token)
        .map(|x| x.claims)
        .map_err(|_| AuthError::InternalServerError)?;

    let mut public_key: Option<Vec<u8>> = None;

    if let Some(public_key_b64) = jwt.public_key {
        public_key = Some(
            BASE64_STANDARD
                .decode(public_key_b64.trim())
                .context("Invalid base64 encoding")?,
        );
    }

    let auth_vals = Auth {
        id: jwt.sub,
        session_id: jwt.session_id,
        username: jwt.username,
        roles: jwt.roles,
        public_key,
        private_key: None,
    };

    AUTH_STATE.set(auth_vals);

    if save_secrets(access_token).is_err() {
        panic!("failed to save save refresh_token");
    };

    let _ = AUTHED_CLIENT
        .initialize(GLOBAL_SECRETS.load().access_token.to_string())
        .await;

    Ok(())
}

pub async fn logout() -> Result<(), CoreError> {
    let _ = no_auth_client()
        .post(format!("{}/auth/logout", GLOBAL_CONFIG.load().api_url))
        .send()
        .await
        .map_err(|_| CoreError::Auth(AuthError::RequestFailure))?
        .error_for_status()
        .map_err(|e| match e.status() {
            Some(StatusCode::INTERNAL_SERVER_ERROR) => {
                CoreError::Auth(AuthError::InternalServerError)
            }
            Some(StatusCode::UNAUTHORIZED) => CoreError::Auth(AuthError::Unauthorized),
            _ => CoreError::Auth(AuthError::RequestFailure),
        })?;

    Ok(())
}
