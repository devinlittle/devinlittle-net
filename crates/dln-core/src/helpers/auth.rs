use crate::{auth::AUTH_STATE, error::CoreError};
use crate::{
    auth::{AuthError, AUTHED_CLIENT},
    fs::GLOBAL_CONFIG,
    structs::{ActiveSessions, AdminGlobalMessage, UserRoles},
};

use uuid::Uuid;

pub fn get_username() -> Result<String, CoreError> {
    let auth = AUTH_STATE.get().ok_or(CoreError::NotInitalized)?;
    Ok(auth.username.clone())
}

pub fn get_user_id() -> Result<Uuid, CoreError> {
    let auth = AUTH_STATE.get().ok_or(CoreError::NotInitalized)?;
    Ok(auth.id)
}

pub fn get_session_id() -> Result<Uuid, CoreError> {
    let auth = AUTH_STATE.get().ok_or(CoreError::NotInitalized)?;
    Ok(auth.session_id)
}

pub fn get_user_roles() -> Result<UserRoles, CoreError> {
    let auth = AUTH_STATE.get().ok_or(CoreError::NotInitalized)?;
    Ok(auth.roles.clone())
}

pub async fn get_sessions() -> Result<Vec<ActiveSessions>, CoreError> {
    AUTHED_CLIENT
        .execute(|client| {
            Box::pin(async move {
                client
                    .get(format!("{}/auth/me/sessions", GLOBAL_CONFIG.load().api_url))
                    .send()
                    .await
                    .map_err(|_| CoreError::NotInitalized)
            })
        })
        .await?
        .json::<Vec<ActiveSessions>>()
        .await
        .map_err(|_| CoreError::Auth(AuthError::RequestFailure))
}

pub async fn global_message(params: AdminGlobalMessage) -> Result<(), CoreError> {
    let _ = AUTHED_CLIENT
        .execute(|client| {
            let params = params.clone();
            Box::pin(async move {
                let params = params.clone();
                client
                    .post(format!(
                        "{}/auth/admin/global_message",
                        GLOBAL_CONFIG.load().api_url
                    ))
                    .json::<AdminGlobalMessage>(&params)
                    .send()
                    .await
                    .map_err(|_| CoreError::NotInitalized)
            })
        })
        .await;
    Ok(())
}
