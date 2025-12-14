use chat_lib::prelude::*;
use ratatui::{
    style::Stylize,
    text::{Line, Span},
};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub enum RoomEvent {
    Message(Message),
    UserLeft(Uuid),
    UserJoined(Uuid),
    UserNameChange { from: String, to: String },
}

pub fn zip_spans<'a>(l: Span<'a>, r: Span<'a>, width: u16) -> Line<'a> {
    #[allow(
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap
    )]
    let spaces = (i32::from(width) - 1 - (l.width() as i32) - (r.width() as i32).max(1)) as usize;

    Line::from_iter([l, (" ".repeat(spaces).into()), r])
}

impl RoomEvent {
    pub fn as_line<'a>(&'a self, width: u16, users: &'a Vec<User>) -> Line<'a> {
        match self {
            RoomEvent::Message(m) => {
                if let Some(user) = users.iter().find(|u| u.get_id() == m.get_author()) {
                    zip_spans(
                        Span::from(user.get_name().to_string()),
                        Span::from(m.get_content()),
                        width,
                    )
                } else {
                    zip_spans(
                        Span::from("Loading...").dim(),
                        Span::from(m.get_content()),
                        width,
                    )
                }
            }
            RoomEvent::UserLeft(id) => zip_spans(
                Span::from(
                    users
                        .get_user(id)
                        .map(|u| u.get_name().to_owned())
                        .unwrap_or(id.to_string()),
                ),
                Span::from("left the chat"),
                width,
            )
            .red(),
            RoomEvent::UserJoined(id) => zip_spans(
                Span::from(
                    users
                        .get_user(id)
                        .map(|u| u.get_name().to_owned())
                        .unwrap_or(id.to_string()),
                ),
                Span::from("joined the chat"),
                width,
            )
            .light_green(),
            RoomEvent::UserNameChange { from, to } => zip_spans(
                Span::from(from.to_owned()),
                Span::from(format!("is now known as {to}")),
                width,
            )
            .light_green(),
        }
    }
}

impl From<Message> for RoomEvent {
    fn from(msg: Message) -> Self {
        Self::Message(msg)
    }
}
