use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::Namespaces;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "smalltalk", derive(sqlx::FromRow))]
pub struct SmalltalkNote {
    pub id: Uuid,
    pub user_id: Uuid,
    pub group_id: Option<Uuid>,

    // using Vec<u8> for BYTEA columns
    #[schema(value_type = String, format = Binary)]
    pub enc_name: Vec<u8>,

    #[schema(value_type = Option<String>, format = Binary)]
    pub enc_content: Option<Vec<u8>>,

    // Password Protection
    pub is_protected: bool,
    pub password_hash: Option<String>,
    #[schema(value_type = Option<String>, format = Binary)]
    pub salt: Option<Vec<u8>>,

    // Metadata and da UI
    pub rank: i32,
    pub is_deleted: bool,
    /// Unix timestamp in milliseconds
    #[schema(value_type = i64, example = 1715760000000_i64)]
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub updated_at: DateTime<Utc>,
    /// Unix timestamp in milliseconds
    #[schema(value_type = i64, example = 1715760000000_i64)]
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "smalltalk", derive(sqlx::FromRow))]
pub struct SmalltalkNotesGroup {
    pub id: Uuid,
    pub user_id: Uuid,

    // using Vec<u8> for BYTEA columns
    #[schema(value_type = String, format = Binary)]
    pub enc_group_name: Vec<u8>,

    // TODO: perhaps have the value type as a struct?
    // but i havent decided concrete on what the metadata could contain so il update later
    #[schema(value_type = Option<String>, format = Binary)]
    pub enc_group_metadata: Option<Vec<u8>>,

    pub rank: i32,
    pub is_deleted: bool,
    /// Unix timestamp in milliseconds
    #[schema(value_type = i64, example = 1715760000000_i64)]
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub updated_at: DateTime<Utc>,
    /// Unix timestamp in milliseconds
    #[schema(value_type = i64, example = 1715760000000_i64)]
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SmalltalkNotesSyncResponse {
    pub notes: Vec<SmalltalkNote>,
    pub groups: Vec<SmalltalkNotesGroup>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct NotePatchRequest {
    #[schema(value_type = Option<String>, format = Binary)]
    pub enc_name: Option<Vec<u8>>,
    #[schema(value_type = Option<String>, format = Binary)]
    pub enc_content: Option<Vec<u8>>,
    pub is_protected: Option<bool>,
    pub password_hash: Option<String>,
    #[schema(value_type = Option<String>, format = Binary)]
    pub salt: Option<Vec<u8>>,
    pub rank: Option<i32>,
    pub is_pinned: Option<bool>,
    pub is_deleted: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct NoteCreateRequest {
    #[schema(value_type = Option<String>, format = Binary)]
    pub enc_name: Vec<u8>,
    pub is_protected: bool,
    pub password_hash: Option<String>,
    #[schema(value_type = Option<String>, format = Binary)]
    pub salt: Option<Vec<u8>>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct NoteGroupCreateRequest {
    #[schema(value_type = Option<String>, format = Binary)]
    pub enc_group_name: Vec<u8>,
    // WARN: read line 51-52
    #[schema(value_type = Option<String>, format = Binary)]
    pub enc_group_metadata: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SmalltalkNotesMessage {
    pub id: Uuid,
    pub namespace: Namespaces,
    pub payload: SmalltalkNotesEvent,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", content = "data")]
pub enum SmalltalkNotesEvent {
    #[schema(title = "NoteAdded")]
    NoteAdded { note: SmalltalkNote },
    #[schema(title = "NoteUpdated")]
    NoteUpdated { note_id: Uuid, note: SmalltalkNote },
    #[schema(title = "NoteDeleted")]
    NoteDeleted { note_id: Uuid },
    #[schema(title = "NoteForgotten")]
    NoteForgotten { note_id: Uuid },

    #[schema(title = "GroupCreated")]
    GroupCreated { group: SmalltalkNotesGroup },
    #[schema(title = "GroupUpdated")]
    GroupUpdated {
        group_id: Uuid,
        group: SmalltalkNotesGroup,
    },
    #[schema(title = "GroupDeleted")]
    GroupDeleted { group_id: Uuid },
}
