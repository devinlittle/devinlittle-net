use std::sync::{Arc, LazyLock};

use os_info::{self, Type};
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue, ACCEPT, CONTENT_TYPE, USER_AGENT},
    Client,
};

use crate::{error::CoreError, fs::COOKIE_STORE};

pub mod auth;
pub mod error;
pub mod fs;
pub mod helpers;
pub mod structs;

pub fn get_user_agent() -> String {
    let app_version = env!("CARGO_PKG_VERSION");
    let info = os_info::get();

    let os_type = info.os_type();

    let distro_name = match os_type {
        Type::Linux => None,
        _ => Some(format!("{:?}", os_type)),
    };

    let mut distro_parts = vec![];

    if let Some(name) = distro_name {
        distro_parts.push(name);
    } else if let Some(codename) = info.codename() {
        distro_parts.push(codename.to_string());
    }

    match info.version() {
        os_info::Version::Semantic(major, minor, patch) => {
            distro_parts.push(format!("{}.{}.{}", major, minor, patch));
        }
        os_info::Version::Custom(v) | os_info::Version::Rolling(Some(v)) => {
            distro_parts.push(v.clone());
        }
        _ => {}
    }

    let distro = distro_parts.join(" ");

    let arch = info.architecture().unwrap_or("");

    let os_part = if is_linux_family(os_type) {
        if distro.trim().is_empty() {
            "Linux".to_string()
        } else {
            format!("Linux [{}]", distro.trim())
        }
    } else if distro.trim().is_empty() {
        format!("{:?}", os_type)
    } else {
        format!("{:?} [{}]", os_type, distro.trim())
    };

    let os_part = if !arch.is_empty() {
        format!("{} {}", os_part, arch)
    } else {
        os_part
    };

    format!(
        "dln-core/{} ({}; Rust/{}) (+https://devinlittle.net)",
        app_version,
        os_part,
        env!("CARGO_PKG_RUST_VERSION")
    )
}

fn is_linux_family(t: Type) -> bool {
    !matches!(
        t,
        Type::Macos
            | Type::Windows
            | Type::Emscripten
            | Type::FreeBSD
            | Type::OpenBSD
            | Type::NetBSD
            | Type::DragonFly
            | Type::MidnightBSD
            | Type::Unknown
    )
}

static REQWEST_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    let mut headers = HeaderMap::new();

    let user_agent = get_user_agent();

    headers.insert(USER_AGENT, HeaderValue::from_str(&user_agent).unwrap());

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

    println!("[dln-core]: successfully loaded!");
    Ok(())
}
