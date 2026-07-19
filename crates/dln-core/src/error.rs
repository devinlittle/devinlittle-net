use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreError {
    /// The core library hasn't been initialized or bootstrap failed
    #[error("Accessing the auth state isn't possible until auth is initalized")]
    NotInitalized,

    #[error("A configuration error; missing variables, invalid URLs, ect")]
    ConfigError(String),

    #[error("network request failed")]
    RequestFailure,

    #[error("Authentication Error")]
    Auth(crate::auth::AuthError),

    #[error("Error connecting to WS")]
    Ws(crate::ws::WsError),

    #[error("filesystem or IO error")]
    Io(std::io::Error),
}
