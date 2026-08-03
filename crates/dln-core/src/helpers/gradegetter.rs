use crate::{
    error::CoreError,
    services::gradegetter::GRADE_STATE,
    structs::{GradesHashMap, SchoologyLogin},
};

use anyhow::Result;
use std::sync::Arc;

pub fn grab_grades() -> Result<Arc<GradesHashMap>, CoreError> {
    let grades = GRADE_STATE.get().ok_or(CoreError::NotInitalized)?;
    Ok(grades)
}

pub async fn add_schoology_credentials(params: SchoologyLogin) -> Result<(), CoreError> {
    crate::services::gradegetter::add_schoology_credentials(params).await?;
    crate::services::gradegetter::forward_to_gradegetter().await?;
    Ok(())
}

pub use crate::services::gradegetter::delete_credentials;
