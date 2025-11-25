use crate::types::{ClientMessage, ServerMessage};

impl ClientMessage {
    pub fn as_wsmsg(&self) -> rocket_ws::Message {
        rocket_ws::Message::Text(self.as_json())
    }
}

impl ServerMessage {
    pub fn as_wsmsg(&self) -> rocket_ws::Message {
        rocket_ws::Message::Text(self.as_json())
    }
}
