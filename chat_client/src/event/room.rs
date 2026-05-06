use chat_lib::prelude::*;
use ratatui::style::Style;
use uuid::Uuid;

use crate::event::{EventType, MessageTrait, UserEventType, UserLocator};

#[derive(Debug, Clone)]
pub enum RoomEvent {
    Message(Message),
    UserLeft(Uuid),
    UserJoined(Uuid),
    UserNameChange { from: String, to: String },
}

impl RoomEvent {
    #[must_use]
    pub fn properties(&self, users: &[User]) -> EventType {
        match self {
            RoomEvent::Message(msg) => EventType::User(UserEventType {
                display_as_loading: true,
                user_uuid: *msg.get_author(),
                user: msg.get_author_from(&users).cloned(),
                message: msg.get_content().to_string(),
                user_style: Style::new().dim(),
                message_style: Style::new(),
            }),
            RoomEvent::UserLeft(uuid) => {
                let style = Style::new().red();
                EventType::User(UserEventType {
                    // this can't be fetched after the user leaves
                    display_as_loading: false,
                    user_uuid: *uuid,
                    user: users.get_user(*uuid).cloned(),
                    message: "left the chat".to_string(),
                    user_style: style,
                    message_style: style,
                })
            }
            RoomEvent::UserJoined(uuid) => {
                let style = Style::new().light_green();
                EventType::User(UserEventType {
                    display_as_loading: true,
                    user_uuid: *uuid,
                    user: users.get_user(*uuid).cloned(),
                    message: "joined the chat".to_string(),
                    user_style: style,
                    message_style: style,
                })
            }
            RoomEvent::UserNameChange { from, to } => EventType::Info {
                message: format!("{from} is now known as {to}"),
                style: Style::new().light_green(),
            },
        }
    }
}

impl From<Message> for RoomEvent {
    fn from(msg: Message) -> Self {
        Self::Message(msg)
    }
}
