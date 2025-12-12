use serde::{Deserialize, Serialize};
use uuid::Uuid;

use std::{sync::Arc, time::Duration};
use tokio::sync::Mutex;

pub type Sync<T> = Arc<Mutex<T>>;

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct User {
    id: Uuid,
    name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct Message {
    from: Uuid,
    content: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum ServerMessage {
    NewMessage(Message),
    AllUsers(Vec<User>),
    UserNameChange(User),
    UserJoined(User),
    UserLeft(User),
    UserData(User),
    SelfData(User),
    UnsupportedMessage(String),
    InvalidUser(Uuid),
    NameTooLong(String),
    Banned { duration: Duration, reason: String },
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum ClientMessage {
    ChangeUserName(String),
    SendMessage(String),
    GetUserData(Uuid),
    GetSelf,
}

impl ServerMessage {
    pub fn as_json(&self) -> String {
        serde_json::to_string(self).expect("Serialize implementation failed")
    }

    pub fn is_user(&self, id: Uuid) -> bool {
        match self {
            ServerMessage::NewMessage(message) => *message.get_author() == id,
            ServerMessage::UserNameChange(user) => *user.get_id() == id,
            ServerMessage::UserJoined(user) => *user.get_id() == id,
            ServerMessage::UserLeft(user) => *user.get_id() == id,
            ServerMessage::UserData(user) => *user.get_id() == id,
            ServerMessage::SelfData(user) => *user.get_id() == id,
            ServerMessage::InvalidUser(uuid) => *uuid == id,
            _ => false,
        }
    }
}

impl ClientMessage {
    pub fn as_json(&self) -> String {
        serde_json::to_string(self).expect("Serialize implementation failed")
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

    pub fn set_name(&mut self, name: String) {
        self.name = name;
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
