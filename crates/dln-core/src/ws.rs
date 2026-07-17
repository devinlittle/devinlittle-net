use self::WsError::*;
use crate::auth::AUTH_STATE;
use crate::fs::GLOBAL_CONFIG;
use crate::fs::GLOBAL_SECRETS;
use crate::services::notification::handle_notification;
use crate::structs::Bootstrap;
use crate::structs::Namespaces;
use crate::structs::{NanoPassPayload, NotificationMessage};
use crate::ws::SocketStates::{Connected, Connecting, Disconnected};
use crate::CoreError;

use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use std::sync::{LazyLock, Mutex};
use thiserror::Error;
use tokio::task::JoinHandle;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message;

#[derive(Error, Debug)]
pub enum WsError {
    #[error("Failed to connect to WS")]
    ConnectionFaliure,

    #[error("The payload provided is invalid")]
    InvalidPayload,

    #[error("The SOCKET static mutex was poisoned")]
    PoisonedMutex,
}

#[derive(Deserialize, Debug)]
pub struct IncomingMessage {
    namespace: Namespaces,
    payload: IncomingPayload,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum IncomingPayload {
    Notification(NotificationMessage),
    NanoPass(NanoPassPayload),
}

pub fn handle_message(msg: IncomingMessage) -> Result<(), CoreError> {
    match msg.namespace {
        Namespaces::Notification => {
            if let IncomingPayload::Notification(payload) = msg.payload {
                handle_notification(payload)?
            } else {
                return Err(CoreError::Ws(InvalidPayload));
            }
        }
        Namespaces::NanoPass => {}
        Namespaces::GradeGetter => (), // Just run FetchGrades Function
        Namespaces::SmallTalkKeySync => (),
        Namespaces::SmallTalkNotes => (),
    }
    Ok(())
}

#[derive(PartialEq, Eq, Clone)]
pub enum SocketStates {
    Connected,
    Connecting,
    Disconnected,
}

pub struct WebsocketManager {
    socket: Option<JoinHandle<()>>,
    state: SocketStates,
}
impl Default for WebsocketManager {
    fn default() -> Self {
        Self {
            socket: None,
            state: SocketStates::Disconnected,
        }
    }
}

pub static SOCKET: LazyLock<Mutex<WebsocketManager>> =
    LazyLock::new(|| Mutex::new(WebsocketManager::default()));

pub fn update_socket_state(state: SocketStates) -> Result<(), CoreError> {
    SOCKET
        .lock()
        .map_err(|_| CoreError::Ws(PoisonedMutex))?
        .state = state;

    Ok(())
}

pub async fn connect_notifications() -> Result<(), CoreError> {
    {
        let should_disconnect = {
            let socket = SOCKET.lock().map_err(|_| CoreError::Ws(PoisonedMutex))?;

            socket.state == Connected || socket.state == Connecting
        };

        if should_disconnect {
            disconnect_notifications()?;
        }
    }
    update_socket_state(Connecting)?;

    let auth = AUTH_STATE.get().ok_or(CoreError::NotInitalized);

    let path = if let Ok(ref auth) = auth {
        format!("/ws/{}", auth.id)
    } else {
        String::from("/ws/global")
    };

    drop(auth);

    let ws_url = format!(
        "{}/notification{}",
        GLOBAL_CONFIG
            .load()
            .api_url
            .replace("https://", "wss://")
            .replace("http://", "ws://"),
        path
    )
    .into_client_request()
    .map_err(|_| CoreError::Ws(ConnectionFaliure))?;

    let (mut stream, _response) = connect_async(ws_url)
        .await
        .map_err(|_| CoreError::Ws(ConnectionFaliure))?;

    update_socket_state(Connected)?;

    let access_token = GLOBAL_SECRETS.load().access_token.clone();

    let bootstrap_json = serde_json::to_value::<Bootstrap>(Bootstrap {
        token: access_token,
    })
    .map_err(|_| CoreError::NotInitalized)?;
    let bootstrap_text =
        serde_json::to_string(&bootstrap_json).map_err(|_| CoreError::NotInitalized)?;

    let bootstrap_text = format!("BOOTSTRAP:{}", bootstrap_text);

    stream
        .send(Message::Text(bootstrap_text.into()))
        .await
        .map_err(|_| CoreError::Ws(InvalidPayload))?;

    let handler = tokio::spawn(async move {
        let mut stream = stream;
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if text.as_str() == "Channel Created" {
                        continue;
                    }

                    let incoming_message = serde_json::from_str::<IncomingMessage>(text.as_str())
                        .map_err(|_| CoreError::Ws(InvalidPayload));

                    if let Ok(incoming) = incoming_message {
                        if handle_message(incoming).is_err() {
                            continue;
                        }
                    } else {
                        continue;
                    }
                }
                Ok(Message::Close(_)) => {
                    println!("Connection closed");
                    break;
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    break;
                }
                _ => {}
            }
        }
    });

    let mut socket = SOCKET.lock().map_err(|_| CoreError::Ws(PoisonedMutex))?;
    socket.socket = Some(handler);
    drop(socket);

    Ok(())
}

fn disconnect_notifications() -> Result<(), CoreError> {
    let handle = {
        let mut socket = SOCKET.lock().map_err(|_| CoreError::Ws(PoisonedMutex))?;

        socket.state = Disconnected;
        socket.socket.take()
    };

    if let Some(joinie) = handle {
        joinie.abort();
    }
    Ok(())
}
