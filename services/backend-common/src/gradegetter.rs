// used in gradegetter_backend/routes/auth.rs

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct SchoologyLogin {
    #[schema(example = "email@exmaple.com")]
    pub schoology_email: String,
    #[schema(example = "password")]
    pub schoology_password: String,
}

// used in gradegetter_backend/routes/internal.rs
#[derive(Deserialize, ToSchema)]
pub struct ForwardMessage {
    pub id: Uuid,
    pub status: ForwardStatus,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub enum GradeGetterPayload {
    GradesUpdated,
}

#[derive(Serialize, Deserialize, ToSchema, Debug, Clone, PartialEq, Display, EnumString)]
pub enum ForwardStatus {
    #[serde(rename = "Started,1")]
    Started,
    #[serde(rename = "Navigated to Schoology login,2")]
    Navigated,
    #[serde(rename = "Typed in Email,3")]
    TypedEmail,
    #[serde(rename = "Entered Email,4")]
    EnteredEmail,
    #[serde(rename = "Typed in Password,5")]
    TypedPassword,
    #[serde(rename = "Enter Password,6")]
    EnteredPassword,
    #[serde(rename = "Finished,7")]
    Finished,
    #[serde(rename = "Incorrect Email or Password,E")]
    ErrorInSetup,
}

impl ForwardStatus {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Started => "1",
            Self::Navigated => "2",
            Self::TypedEmail => "3",
            Self::EnteredEmail => "4",
            Self::TypedPassword => "5",
            Self::EnteredPassword => "6",
            Self::Finished => "7",
            Self::ErrorInSetup => "E",
        }
    }
}

impl ForwardStatus {
    pub fn ws_from_str(s: &str) -> Option<Self> {
        match s.trim() {
            "Started,1" => Some(ForwardStatus::Started),
            "Navigated to Schoology login,2" => Some(ForwardStatus::Navigated),
            "Typed in Email,3" => Some(ForwardStatus::TypedEmail),
            "Entered Email,4" => Some(ForwardStatus::EnteredEmail),
            "Typed in Password,5" => Some(ForwardStatus::TypedPassword),
            "Enter Password,6" => Some(ForwardStatus::EnteredPassword),
            "Finished,7" => Some(ForwardStatus::Finished),
            "Incorrect Email or Password,E" => Some(ForwardStatus::ErrorInSetup),
            _ => None,
        }
    }
}

pub type GradesHashMap = BTreeMap<String, Vec<Option<f32>>>;
