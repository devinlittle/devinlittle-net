use crate::{error::CoreError, structs::NotificationMessage};

pub fn handle_notification(payload: NotificationMessage) -> Result<(), CoreError> {
    println!("got payload: {:?}", payload);
    Ok(())
}
