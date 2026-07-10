use std::sync::{Arc, LazyLock};

use reqwest::{
    Client,
    header::{ACCEPT, CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue, USER_AGENT},
};

use crate::{error::CoreError, fs::COOKIE_STORE};

pub mod auth;
pub mod error;
pub mod fs;
pub mod helpers;
pub mod structs;

static REQWEST_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    let mut headers = HeaderMap::new();

    let version = env!("CARGO_PKG_VERSION");
    let user_agent_string = format!("dln-core/{} (+https://devinlittle.net)", version);

    headers.insert(
        USER_AGENT,
        HeaderValue::from_str(&user_agent_string).unwrap(),
    );

    headers.insert(
        HeaderName::from_static("x-dln-client-version"),
        HeaderValue::from_str(env!("CARGO_PKG_VERSION")).unwrap(),
    );

    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    Client::builder()
        .default_headers(headers)
        .cookie_provider(Arc::clone(&COOKIE_STORE))
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("Failed to build global reqwest client")
});

pub fn no_auth_client() -> &'static Client {
    &REQWEST_CLIENT
}

pub async fn init() -> Result<(), CoreError> {
    if fs::init_directories().is_err() {
        panic!("failed to setup dln dirs");
    }

    fs::load_config_from_disk()?;
    fs::load_secrets_from_disk()?;

    auth::get_ready_for_devin_grfd(false).await?;

    Ok(())
}
