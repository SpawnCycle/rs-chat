use rocket::serde::{Deserialize, Serialize};
use uuid::Uuid;

use std::sync::Arc;
use tokio::sync::Mutex;

pub type Sync<T> = Arc<Mutex<T>>;

#[derive(Clone, Serialize, Deserialize)]
pub struct User {
    id: Uuid,
    name: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Message {
    from: Uuid,
    content: String,
}

/// collection of all the types that may fly between parts of both the server and the client
#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum WsMessage {
    NewMessage(Message),
    GetAllUsers(Vec<User>),
    UserLeft(User),
    UserJoined(User),
    UserNameChange(User),
    GetUserName(Uuid),
    Arbitrary(String),
    UnsupportedMessage,
}

impl WsMessage {
    pub fn is_user(&self, id: Uuid) -> Option<bool> {
        match self {
            WsMessage::NewMessage(msg) => Some(*msg.get_author() == id),
            WsMessage::UserLeft(user) => Some(*user.get_id() == id),
            WsMessage::UserJoined(user) => Some(*user.get_id() == id),
            WsMessage::UserNameChange(user) => Some(*user.get_id() == id),
            _ => None,
        }
    }

    pub fn as_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    pub fn as_wsmsg(&self) -> rocket_ws::Message {
        rocket_ws::Message::Text(self.as_json())
    }
}

impl User {
    pub fn new(id: Uuid, name: String) -> Self {
        Self { id, name }
    }

    pub fn get_id(&self) -> &Uuid {
        &self.id
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }
}

impl Message {
    pub fn new(from: Uuid, content: String) -> Self {
        Self { from, content }
    }

    pub fn get_content(&self) -> &str {
        &self.content
    }

    pub fn get_author(&self) -> &Uuid {
        &self.from
    }
}
