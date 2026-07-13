pub use backend_common::auth::ActiveSessions;
pub use backend_common::auth::LoginInput as LoginPayload;
pub use backend_common::ServiceName;
pub use backend_common::UserRole;
pub use backend_common::UserRoles;

use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct AdminGlobalMessage {
    pub content: String,
    pub title: String,
}
