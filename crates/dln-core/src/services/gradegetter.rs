use crate::auth::AUTH_STATE;
use crate::event_bus::{EventBusEvent, EVENT_BUS};
use crate::fs::GLOBAL_CONFIG;
use crate::services::gradegetter::GradeOutput::{BTreeGrades, JsonGrades};
use crate::structs::{ForwardStatus, GradesHashMap, SchoologyLogin};
use crate::{auth::AUTHED_CLIENT, error::CoreError};

use anyhow::Result;
use arc_swap::ArcSwap;
use futures_util::StreamExt;
use serde_json::Value;
use std::sync::{Arc, LazyLock};
use tokio::sync::watch::{self, Receiver};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message;

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

pub enum GradeOutput {
    BTreeGrades(Grades),
    JsonGrades(Value),
}

pub async fn fetch_grades(json: bool) -> Result<GradeOutput, CoreError> {
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

    match json {
        true => {
            let grades = serde_json::to_value(grades).map_err(|_| CoreError::RequestFailure)?;
            Ok(GradeOutput::JsonGrades(grades))
        }
        false => Ok(GradeOutput::BTreeGrades(grades)),
    }
}

pub async fn set_grades() -> Result<(), CoreError> {
    let grades = fetch_grades(false).await?;
    let grades = match grades {
        GradeOutput::BTreeGrades(grades) => Ok(grades),
        GradeOutput::JsonGrades(_) => Err("Expected BTreeGrades, got JsonGrades".to_string()),
    }
    .map_err(|_| CoreError::RequestFailure)?;

    GRADE_STATE.set(grades);
    if let Some(tx) = EVENT_BUS.get() {
        tx.send(EventBusEvent::GradesUpdated)
            .await
            .unwrap_or_default();
    }
    Ok(())
}

pub async fn add_schoology_credentials(params: SchoologyLogin) -> Result<(), CoreError> {
    let _ = AUTHED_CLIENT
        .execute(|client| {
            let params = params.clone();
            Box::pin(async move {
                client
                    .post(format!(
                        "{}/gradegetter/auth/schoology/credentials",
                        GLOBAL_CONFIG.load().api_url
                    ))
                    .json::<SchoologyLogin>(&params)
                    .send()
                    .await
                    .map_err(|_| CoreError::NotInitalized)
            })
        })
        .await
        .map_err(|_| CoreError::RequestFailure)?;
    Ok(())
}

pub async fn del_schoology_credentials() -> Result<(), CoreError> {
    let _ = AUTHED_CLIENT
        .execute(|client| {
            Box::pin(async move {
                client
                    .delete(format!(
                        "{}/gradegetter/auth/schoology/credentials",
                        GLOBAL_CONFIG.load().api_url
                    ))
                    .send()
                    .await
                    .map_err(|_| CoreError::NotInitalized)
            })
        })
        .await
        .map_err(|_| CoreError::RequestFailure)?;
    Ok(())
}

pub async fn forward_to_gradegetter() -> Result<(), CoreError> {
    let _ = AUTHED_CLIENT
        .execute(|client| {
            Box::pin(async move {
                client
                    .get(format!(
                        "{}/gradegetter/auth/forward",
                        GLOBAL_CONFIG.load().api_url
                    ))
                    .send()
                    .await
                    .map_err(|_| CoreError::NotInitalized)
            })
        })
        .await
        .map_err(|_| CoreError::RequestFailure)?;
    Ok(())
}
pub async fn forward_ws() -> Result<Receiver<ForwardStatus>, CoreError> {
    let auth = AUTH_STATE.get().ok_or(CoreError::NotInitalized)?;
    let (tx, rx) = watch::channel(ForwardStatus::Started);

    tokio::spawn(async move {
        let tx = tx;

        let Ok(ws_url) = format!(
            "{}/gradegetter/auth/forward_ws/{}",
            GLOBAL_CONFIG
                .load()
                .api_url
                .replace("https://", "wss://")
                .replace("http://", "ws://"),
            auth.id
        )
        .into_client_request() else {
            return;
        };

        let ws_stream = match connect_async(ws_url).await {
            Ok(ws_stream) => ws_stream,
            Err(_) => {
                return;
            }
        };

        let (mut ws_stream, _) = ws_stream;

        while let Some(msg) = ws_stream.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    let Some(status) = ForwardStatus::ws_from_str(text.trim()) else {
                        let _ = tx.send(ForwardStatus::ErrorInSetup);
                        return;
                    };

                    let Ok(()) = tx.send(status) else {
                        let _ = tx.send(ForwardStatus::ErrorInSetup);
                        return;
                    };

                    continue;
                }
                Ok(Message::Close(_)) => {
                    let _ = tx.send(ForwardStatus::ErrorInSetup);
                    break;
                }
                Err(_) => {
                    let _ = tx.send(ForwardStatus::ErrorInSetup);
                    break;
                }
                _ => {}
            }
        }
    });

    Ok(rx)
}
