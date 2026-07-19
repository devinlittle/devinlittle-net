use crate::{error::CoreError, services::gradegetter::GRADE_STATE, structs::GradesHashMap};

use anyhow::Result;
use std::sync::Arc;

pub fn grab_grades() -> Result<Arc<GradesHashMap>, CoreError> {
    let grades = GRADE_STATE.get().ok_or(CoreError::NotInitalized)?;
    Ok(grades)
}
