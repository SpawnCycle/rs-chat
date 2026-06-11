use crate::types::{ClientMessage, ServerMessage};
use axum::extract::ws::Message;

impl ClientMessage {
    #[must_use]
    pub fn as_wsmsg(&self) -> Message {
        Message::text(self.as_json())
    }
}

impl ServerMessage {
    #[must_use]
    pub fn as_wsmsg(&self) -> Message {
        Message::text(self.as_json())
    }
}
