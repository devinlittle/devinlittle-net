use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreError {
    /// The entire core library hasn't been initialized or bootstrap failed
    #[error("Accessing the auth state isn't possible until auth is initalized")]
    NotInitalized,

    #[error("A configuration error; missing variables, invalid URLs, ect")]
    ConfigError(String),

    #[error("Authentication Error")]
    Auth(crate::auth::AuthError),

    #[error("filesystem or IO error")]
    Io(std::io::Error),
}
