use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;
use uuid::Uuid;

#[cfg(feature = "auth")]
pub mod auth;

#[cfg(feature = "gradegetter")]
pub mod gradegetter;

#[cfg(feature = "nanopass")]
pub mod nanopass;

#[cfg(feature = "notifications")]
pub mod notification;

#[cfg(feature = "smalltalk")]
pub mod smalltalk;

#[cfg(feature = "internal")]
pub mod internal;

#[cfg(feature = "tracing")]
pub mod tracing;

#[derive(Clone, ToSchema, Serialize, Deserialize, Debug)]
pub struct AuthenticatedUser {
    pub username: String,
    pub uuid: Uuid,
    pub role: UserRole,
    pub session_id: Uuid,
}

pub type UserRoles = HashMap<ServiceName, UserRole>;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, ToSchema)]
pub struct Claims {
    pub sub: Uuid,
    pub username: String,
    pub roles: UserRoles,
    pub session_id: Uuid,
    pub public_key: Option<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub iat: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub exp: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Eq, Hash, PartialEq, Clone, Debug, ToSchema, strum::Display)]
//#[serde(rename_all = "snake_case")]
#[serde(rename_all = "lowercase")]
pub enum ServiceName {
    /// This can NOT be used in the client
    #[serde(alias = "Auth")]
    Auth,
    /// This can be used in the client
    #[serde(alias = "Global")]
    Global,
    /// This can be used in the client
    #[serde(alias = "Gradegetter")]
    GradeGetter,
    /// This can NOT be used in the client
    #[serde(alias = "GradeGetter_Backend")]
    GradeGetterBackend,
    /// This can be used in the client
    #[serde(alias = "Nanopass")]
    NanoPass,
    /// This can be used in the client
    #[serde(alias = "Smalltalk")]
    SmallTalk,
    /// This can be used in the client
    #[serde(alias = "Notifications")]
    Notifications,
    #[serde(alias = "PodcastSchoolProject")]
    /// This can be used in the client
    PodcastSchoolProject,
    /// This can NOT be used in the client
    #[serde(alias = "Unknown")]
    Unknown,
}

#[derive(Serialize, Deserialize, Eq, Hash, PartialEq, Clone, Debug, ToSchema, strum::Display)]
//#[serde(rename_all = "snake_case")]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Devin,
    Owen,
    MrD,
    Trusted,
    User,
}

impl UserRole {
    pub fn is_admin(&self) -> bool {
        matches!(self, Self::Devin | Self::Owen)
    }
}

#[derive(Serialize, Deserialize, Eq, Hash, PartialEq, Clone, Debug, ToSchema, strum::Display)]
#[serde(rename_all = "lowercase")]
#[schema(rename_all = "lowercase")]
pub enum Namespaces {
    Notification,
    NanoPass,
    GradeGetter,
    #[serde(rename = "smalltalk_keysync")]
    #[schema(rename = "smalltalk_keysync")]
    SmallTalkKeySync,
    #[serde(rename = "smalltalk_notes")]
    #[schema(rename = "smalltalk_notes")]
    SmallTalkNotes,
}
