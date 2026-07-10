use anyhow::{Context, Result};
use arc_swap::ArcSwap;
use directories::ProjectDirs;
use reqwest_cookie_store::{CookieStore, CookieStoreMutex};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::sync::{Arc, LazyLock, OnceLock};

use crate::error::CoreError;

pub static APP_PATHS: OnceLock<AppPaths> = OnceLock::new();

pub static GLOBAL_CONFIG: LazyLock<ArcSwap<AppConfig>> =
    LazyLock::new(|| ArcSwap::from_pointee(AppConfig::default()));
pub static GLOBAL_SECRETS: LazyLock<ArcSwap<TokenSecrets>> =
    LazyLock::new(|| ArcSwap::from_pointee(TokenSecrets::default()));

pub struct AppPaths {
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub config_toml: PathBuf,
    pub sqlite_db: PathBuf,
    pub access_token: PathBuf,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppConfig {
    #[serde(default = "default_api_url")]
    pub api_url: String,
}

fn default_api_url() -> String {
    String::from("https://api.devinlittle.net")
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            api_url: default_api_url(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct TokenSecrets {
    pub access_token: String,
}

pub static COOKIE_STORE: LazyLock<Arc<CookieStoreMutex>> = LazyLock::new(|| {
    let paths = APP_PATHS
        .get()
        .expect("the paths must be initialized before utilizing cookies");
    let cookie_file_path = paths.data_dir.join("cookies.json");

    let raw_store = if let Ok(file) = File::open(&cookie_file_path).map(BufReader::new) {
        cookie_store::serde::json::load(file).unwrap_or_else(|_| CookieStore::new())
    } else {
        CookieStore::new()
    };

    Arc::new(CookieStoreMutex::new(raw_store))
});

pub fn init_directories() -> Result<()> {
    let proj_dirs =
        ProjectDirs::from("net", "devinlittle", "dln").context("failed creating proj_dirs")?;
    let config_dir = proj_dirs.config_dir().to_path_buf();
    let data_dir = proj_dirs.data_dir().to_path_buf();

    fs::create_dir_all(&config_dir)
        .ok()
        .context("failed to create config_dir")?;
    fs::create_dir_all(&data_dir)
        .ok()
        .context("failed to create data_dir")?;

    let paths = AppPaths {
        config_toml: config_dir.join("config.toml"),
        sqlite_db: data_dir.join("dln.db"),
        access_token: data_dir.join("access_token.txt"),
        config_dir,
        data_dir,
    };

    if !paths.config_toml.exists() {
        let default_config = AppConfig::default();

        if let Ok(toml_string) = toml::to_string_pretty(&default_config)
            && let Ok(mut file) = File::create(&paths.config_toml)
        {
            let docs_url = "https://github.com/devinlittle/devinlittle-net.git";
            let file_contents = format!(
                "### DLN configuration!!! ###\n### check docs at {} ###\n\n{}",
                docs_url, toml_string
            );
            file.write_all(file_contents.as_bytes())
                .context("failed to write bytes to config.toml")?;
        }
    }

    let gitignore_path = paths.config_dir.join(".gitignore");
    if !gitignore_path.exists() {
        let mut file = File::create(&gitignore_path)
            .ok()
            .context("failed to create .gitignore")?;
        file.write_all(b"*\n!config.toml\n")
            .ok()
            .context("failed to write bytes to .gitignore")?;
    }

    let _ = APP_PATHS.set(paths);

    Ok(())
}

#[allow(clippy::redundant_closure)]
pub fn load_config_from_disk() -> Result<(), CoreError> {
    let paths = APP_PATHS.get().expect("App paths must be initialized");
    if paths.config_toml.exists() {
        let content = fs::read_to_string(&paths.config_toml).map_err(|e| CoreError::Io(e))?;
        let config: AppConfig = toml::from_str(&content).unwrap_or_else(|_| AppConfig::default());
        GLOBAL_CONFIG.store(Arc::new(config));
    }
    Ok(())
}

#[allow(clippy::redundant_closure)]
pub fn save_config(new_config: AppConfig) -> Result<(), CoreError> {
    let paths = APP_PATHS.get().expect("App paths must be initialized");
    let content =
        toml::to_string_pretty(&new_config).map_err(|e| CoreError::Io(std::io::Error::other(e)))?;
    fs::write(&paths.config_toml, content).map_err(|e| CoreError::Io(e))?;
    GLOBAL_CONFIG.store(Arc::new(new_config));
    Ok(())
}

#[allow(clippy::redundant_closure)]
pub fn load_secrets_from_disk() -> Result<(), CoreError> {
    let paths = APP_PATHS.get().expect("App paths must be initialized");
    let mut secrets = TokenSecrets::default();
    if paths.access_token.exists() {
        secrets.access_token = fs::read_to_string(&paths.access_token)
            .map_err(|e| CoreError::Io(e))?
            .trim()
            .to_string();
    }
    GLOBAL_SECRETS.store(Arc::new(secrets));
    Ok(())
}

#[allow(clippy::redundant_closure)]
pub fn save_secrets(access: String) -> Result<(), CoreError> {
    let paths = APP_PATHS.get().expect("App paths must be initialized");
    fs::write(&paths.access_token, &access).map_err(|e| CoreError::Io(e))?;
    GLOBAL_SECRETS.store(Arc::new(TokenSecrets {
        access_token: access,
    }));
    Ok(())
}

#[allow(clippy::redundant_closure)]
pub fn save_cookies() -> Result<(), CoreError> {
    let paths = APP_PATHS.get().expect("App paths must be initialized");
    let cookie_file_path = paths.data_dir.join("cookies.json");
    let mut file = File::create(cookie_file_path)
        .map(BufWriter::new)
        .map_err(|e| CoreError::Io(e))?;
    let store = COOKIE_STORE.lock().unwrap();
    cookie_store::serde::json::save(&store, &mut file)
        .map_err(|e| CoreError::Io(std::io::Error::other(e)))?;
    Ok(())
}
