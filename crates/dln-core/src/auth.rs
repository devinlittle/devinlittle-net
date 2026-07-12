use anyhow::{Context, Result};
use arc_swap::ArcSwap;
use aws_lc_rs::agreement::PrivateKey;
use backend_common::{
    auth::{LoginInput, LoginOutput},
    Claims, UserRoles,
};
use base64::{prelude::BASE64_STANDARD, Engine};
use reqwest::StatusCode;
use std::sync::{Arc, OnceLock};
use thiserror::Error;
use uuid::Uuid;

use crate::{
    error::CoreError,
    fs::{save_cookies, save_secrets, COOKIE_STORE, GLOBAL_CONFIG, GLOBAL_SECRETS},
    helpers::auth::get_username,
    no_auth_client,
};

#[derive(Clone, Debug)]
pub struct Auth {
    pub id: Uuid,
    pub session_id: Uuid,
    pub username: String,
    pub roles: UserRoles,
    pub public_key: Option<Vec<u8>>,
    pub private_key: Option<Arc<PrivateKey>>,
}

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
}

pub static AUTH_STATE: OnceLock<ArcSwap<Auth>> = OnceLock::new();

pub fn auth_state() -> Result<&'static ArcSwap<Auth>, CoreError> {
    AUTH_STATE.get().ok_or(CoreError::NotInitalized)
}

#[allow(clippy::collapsible_if)]
pub async fn get_ready_for_devin_grfd(called_from_login_page: bool) -> Result<(), CoreError> {
    if !called_from_login_page {
        if refresh().await.is_err() {
            // connect_notifications

            return Err(CoreError::Auth(AuthError::Unauthorized));
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

    if secrets.access_token.is_empty() && !has_cookies {
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

    let _ = set_token(output.access_token);

    Ok(())
}

// TODO: have the openapi doc used instead of hardcoded urls
pub async fn login_req(params: LoginInput) -> Result<(), CoreError> {
    let output = no_auth_client()
        .post(format!("{}/auth/login", GLOBAL_CONFIG.load().api_url))
        .json(&params)
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

    let _ = set_token(output.access_token);

    let _ = get_ready_for_devin_grfd(true).await;

    Ok(())
}

pub fn set_token(access_token: String) -> Result<()> {
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

    let auth_vals = Arc::new(Auth {
        id: jwt.sub,
        session_id: jwt.session_id,
        username: jwt.username,
        roles: jwt.roles,
        public_key,
        private_key: None,
    });

    let state = ArcSwap::new(auth_vals);
    if AUTH_STATE.set(state).is_err() {
        panic!("failed to setup auth");
    }

    if save_secrets(access_token).is_err() {
        panic!("failed to save save refresh_token");
    };

    Ok(())
}
