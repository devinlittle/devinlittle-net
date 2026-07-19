use std::sync::OnceLock;

use tokio::sync::mpsc;

pub static EVENT_BUS: OnceLock<mpsc::Sender<EventBusEvent>> = OnceLock::new();

pub fn init_events(tx: mpsc::Sender<EventBusEvent>) {
    EVENT_BUS.set(tx).expect("already initialized");
}

#[derive(Debug)]
pub enum EventBusEvent {
    GradesUpdated,
}
