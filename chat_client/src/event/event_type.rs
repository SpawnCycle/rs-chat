use chat_lib::prelude::*;
use ratatui::{
    style::Style,
    text::{Line, Span},
};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum EventType {
    /// Notification about an event
    Info { message: String, style: Style },
    /// Any event that involves a user
    User(UserEventType),
}

#[derive(Debug, Clone)]
pub struct UserEventType {
    pub display_as_loading: bool,
    pub user_uuid: Uuid,
    pub user: Option<User>,
    pub message: String,
    pub user_style: Style,
    pub message_style: Style,
}

impl UserEventType {
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
