pub use backend_common::auth::ActiveSessions;
pub use backend_common::auth::LoginInput as LoginPayload;
pub use backend_common::Namespaces;
pub use backend_common::{ServiceName, UserRole, UserRoles};

// Auth
pub use backend_common::{
    auth::{LoginInput, LoginOutput},
    Claims,
};

// GradeGetter
pub use backend_common::gradegetter::GradeGetterPayload;
pub use backend_common::gradegetter::GradesHashMap;

// NanoPass
pub use backend_common::nanopass::{FileListing, FileListingInput, Visibility};
pub use backend_common::nanopass::{NanoPassMessage, NanoPassPayload};
pub use backend_common::nanopass::{RemoveListingInput, RemoveSessionInput};

// Notifications
pub use backend_common::notification::Bootstrap;
pub use backend_common::notification::{NotificationMessage, NotificationType};

use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct AdminGlobalMessage {
    pub content: String,
    pub title: String,
}
