use crate::types::{ClientMessage, ServerMessage};

impl ClientMessage {
    #[must_use]
    pub fn as_wsmsg(&self) -> rocket_ws::Message {
        rocket_ws::Message::Text(self.as_json())
    }
}

impl ServerMessage {
    #[must_use]
    pub fn as_wsmsg(&self) -> rocket_ws::Message {
        rocket_ws::Message::Text(self.as_json())
    }
}
