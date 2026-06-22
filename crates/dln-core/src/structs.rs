use backend_common::UserRoles;
use uuid::Uuid;

#[derive(Clone, Default)]
pub struct Auth {
    pub id: Option<Uuid>,
    pub session_id: Option<Uuid>,
    pub username: Option<String>,
    pub roles: Option<UserRoles>,
    pub public_key: Option<Vec<u8>>,
    pub private_key: Option<Vec<u8>>,
    pub access_token: Option<String>,
    pub ready: bool,
}
