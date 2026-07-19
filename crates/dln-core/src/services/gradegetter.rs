use crate::auth::AuthError::RequestFailure;
use crate::event_bus::{EventBusEvent, EVENT_BUS};
use crate::fs::GLOBAL_CONFIG;
use crate::structs::GradesHashMap;
use crate::{auth::AUTHED_CLIENT, error::CoreError};

use anyhow::Result;
use arc_swap::ArcSwap;
use std::sync::{Arc, LazyLock};

type Grades = GradesHashMap;

pub static GRADE_STATE: LazyLock<GradeState> = LazyLock::new(GradeState::default);

pub struct GradeState {
    inner: ArcSwap<Option<Arc<Grades>>>,
}

impl Default for GradeState {
    fn default() -> Self {
        Self {
            inner: ArcSwap::from_pointee(None),
        }
    }
}

impl GradeState {
    pub fn get(&self) -> Option<Arc<Grades>> {
        let current = self.inner.load();

        match &**current {
            Some(auth) => Some(Arc::clone(auth)),
            None => None,
        }
    }

    pub fn set(&self, grades: Grades) {
        self.inner.store(Arc::new(Some(Arc::new(grades))));
    }

    pub fn clear(&self) {
        self.inner.store(Arc::new(None));
    }
}

async fn fetch_grades() -> Result<Grades, CoreError> {
    let grades = AUTHED_CLIENT
        .execute(|client| {
            Box::pin(async move {
                client
                    .get(format!(
                        "{}/gradegetter/grades",
                        GLOBAL_CONFIG.load().api_url
                    ))
                    .send()
                    .await
                    .map_err(|_| CoreError::NotInitalized)
            })
        })
        .await?
        .json::<Grades>()
        .await
        .map_err(|_| CoreError::RequestFailure)?;
    Ok(grades)
}

pub async fn set_grades() -> Result<(), CoreError> {
    let grades = fetch_grades().await?;
    GRADE_STATE.set(grades);
    if let Some(tx) = EVENT_BUS.get() {
        tx.send(EventBusEvent::GradesUpdated)
            .await
            .unwrap_or_default();
    }
    Ok(())
}
