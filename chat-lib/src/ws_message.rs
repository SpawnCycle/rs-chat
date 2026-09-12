use crate::types::{ClientMessage, ServerMessage};
use tokio_tungstenite::tungstenite::Message;

#[cfg(feature = "server")]
use axum::extract::ws::Message as AxumMessage;

#[cfg(feature = "server")]
impl ClientMessage {
    #[must_use]
    pub fn as_axum_ws(&self) -> AxumMessage {
        AxumMessage::text(self.as_json())
    }
}

#[cfg(feature = "server")]
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
