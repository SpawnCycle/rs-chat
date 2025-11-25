use chat_lib::prelude::*;
use ratatui::{
    style::Stylize,
    widgets::{Cell, Row},
};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub enum RoomEvent {
    Message(Message),
    UserLeft(Uuid),
    UserJoined(Uuid),
    UserNameChange { from: String, to: String },
}

impl RoomEvent {
    pub fn as_row<'a>(&'a self, users: &'a Vec<User>) -> Row<'a> {
        match self {
            RoomEvent::Message(m) => {
                if let Some(user) = users.iter().find(|u| u.get_id() == m.get_author()) {
                    Row::new(vec![
                        Cell::new(user.get_name().to_string()),
                        Cell::new(m.get_content()),
                    ])
                } else {
                    Row::new(vec![
                        Cell::new("Loading...").dim(),
                        Cell::new(m.get_content()),
                    ])
                }
            }
            RoomEvent::UserLeft(id) => Row::new(vec![
                Cell::new(
                    users
                        .get_user(id)
                        .map(|u| u.get_name().to_owned())
                        .unwrap_or(id.to_string()),
                ),
                Cell::new("left the chat"),
            ])
            .red(),
            RoomEvent::UserJoined(id) => Row::new(vec![
                Cell::new(
                    users
                        .get_user(id)
                        .map(|u| u.get_name().to_owned())
                        .unwrap_or(id.to_string()),
                ),
                Cell::new("joined the chat"),
            ])
            .light_green(),
            RoomEvent::UserNameChange { from, to } => Row::new(vec![
                Cell::new(from.to_owned()),
                Cell::new(format!("is now known as {to}")),
            ])
            .light_green(),
        }
    }
}

impl From<Message> for RoomEvent {
    fn from(msg: Message) -> Self {
        Self::Message(msg)
    }
}

pub fn message(msg: Message) -> RoomEvent {
    RoomEvent::Message(msg)
}

pub fn left(usr: Uuid) -> RoomEvent {
    RoomEvent::UserLeft(usr)
}

pub fn joined(usr: Uuid) -> RoomEvent {
    RoomEvent::UserJoined(usr)
}

pub fn name_change(from: String, to: String) -> RoomEvent {
    RoomEvent::UserNameChange { from, to }
}
