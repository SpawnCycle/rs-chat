use crate::types::{ClientMessage, ServerMessage};
use axum::extract::ws::Message as AxumMessage;
use tokio_tungstenite::tungstenite::Message;

impl ClientMessage {
    #[must_use]
    pub fn as_axum_ws(&self) -> AxumMessage {
        AxumMessage::text(self.as_json())
    }
}

impl ServerMessage {
    #[must_use]
    pub fn as_axum_ws(&self) -> AxumMessage {
        AxumMessage::text(self.as_json())
    }
}

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
