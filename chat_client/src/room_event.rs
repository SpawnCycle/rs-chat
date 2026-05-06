use chat_lib::prelude::*;
use ratatui::{
    style::Style,
    text::{Line, Span},
};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum RoomEvent {
    Message(Message),
    UserLeft(Uuid),
    UserJoined(Uuid),
    UserNameChange { from: String, to: String },
}

#[derive(Debug, Clone)]
pub enum EventProperties {
    /// Notification about an event
    Info { message: String, style: Style },
    /// Any event that involves a user
    User(UserEventProperties),
}

#[derive(Debug, Clone)]
pub struct UserEventProperties {
    pub display_as_loading: bool,
    pub user_uuid: Uuid,
    pub user: Option<User>,
    pub message: String,
    pub user_style: Style,
    pub message_style: Style,
}

impl UserEventProperties {
    #[must_use]
    pub fn user_width(&self) -> usize {
        let user_string = match &self.user {
            Some(user) => user.get_name().to_string(),
            None if self.display_as_loading => "Loading...".to_string(),
            None => self.user_uuid.to_string(),
        };

        user_string.chars().count()
    }

    #[must_use]
    pub fn build_lines(&self, max_user_width: usize, overall_width: usize) -> Vec<Line<'_>> {
        let mut rows = Vec::new();

        let user_string = match &self.user {
            Some(user) => user.get_name().to_string(),
            None if self.display_as_loading => "Loading...".to_string(),
            None => self.user_uuid.to_string(),
        };
        let user_string_width = user_string.chars().count();
        let message_chars = self.message.chars().collect::<Vec<_>>();

        // the full width of the message
        let event_width = 1 + user_string_width + message_chars.len();
        let full_message_width = event_width - max_user_width;
        let message_width = full_message_width.min(overall_width - max_user_width);

        let mut message_parts = message_chars.chunks(message_width);

        // process the first row of the message
        let first_message = String::from_iter(message_parts.next().unwrap_or(&[]));
        let first_line = Line::from_iter([
            Span::from(user_string).style(self.user_style),
            Span::from(" ").style(self.user_style),
            Span::from(first_message).style(self.message_style),
        ]);
        rows.push(first_line);

        // process the remaining parts of the message
        for message in message_parts {
            let message = String::from_iter(message);
            let line = Line::from_iter([
                Span::from(" ".repeat(max_user_width)).style(self.user_style),
                Span::from(message).style(self.message_style),
            ]);
            rows.push(line);
        }

        rows
    }
}

impl RoomEvent {
    #[must_use]
    pub fn properties(&self, users: &[User]) -> EventProperties {
        match self {
            RoomEvent::Message(msg) => EventProperties::User(UserEventProperties {
                display_as_loading: true,
                user_uuid: *msg.get_author(),
                user: msg.get_author_user(&users),
                message: msg.get_content().to_string(),
                user_style: Style::new().dim(),
                message_style: Style::new(),
            }),
            RoomEvent::UserLeft(uuid) => {
                let style = Style::new().red();
                EventProperties::User(UserEventProperties {
                    // this can't be fetched after the user leaves
                    display_as_loading: false,
                    user_uuid: *uuid,
                    user: users.get_user(uuid),
                    message: "left the chat".to_string(),
                    user_style: style,
                    message_style: style,
                })
            }
            RoomEvent::UserJoined(uuid) => {
                let style = Style::new().light_green();
                EventProperties::User(UserEventProperties {
                    display_as_loading: true,
                    user_uuid: *uuid,
                    user: users.get_user(uuid),
                    message: "joined the chat".to_string(),
                    user_style: style,
                    message_style: style,
                })
            }
            RoomEvent::UserNameChange { from, to } => EventProperties::Info {
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
