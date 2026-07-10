use crate::structs::UserRoles;
use uuid::Uuid;

use crate::{auth::auth_state, error::CoreError};

pub fn get_username() -> Result<String, CoreError> {
    let auth = auth_state()?.load();
    Ok(auth.username.clone())
}

pub fn get_user_id() -> Result<Uuid, CoreError> {
    let auth = auth_state()?.load();
    Ok(auth.id)
}

pub fn get_session_id() -> Result<Uuid, CoreError> {
    let auth = auth_state()?.load();
    Ok(auth.session_id)
}

pub fn get_user_roles() -> Result<UserRoles, CoreError> {
    let auth = auth_state()?.load();
    Ok(auth.roles.clone())
}
