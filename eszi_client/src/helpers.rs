use crate::room_event::RoomEvent;
use eszi_lib::types::*;
use ratatui::{style::Stylize, text::Span};
use uuid::Uuid;

pub trait AsSpan {
    fn as_span(&self) -> Span<'_>;
}

pub trait FindUser {
    fn get_user<T>(&self, id: T) -> Option<&User>
    where
        T: AsRef<Uuid>;
}

impl AsSpan for Option<&User> {
    fn as_span(&self) -> Span<'_> {
        match self {
            Some(usr) => usr.as_span(),
            None => Span::from("Loading...").dim(),
        }
    }
}

impl AsSpan for Option<User> {
    fn as_span(&self) -> Span<'_> {
        match self {
            Some(usr) => usr.as_span(),
            None => Span::from("Loading...").dim(),
        }
    }
}

impl AsSpan for &User {
    fn as_span(&self) -> Span<'_> {
        Span::from(self.get_name())
    }
}

impl AsSpan for User {
    fn as_span(&self) -> Span<'_> {
        Span::from(self.get_name())
    }
}

impl FindUser for &Vec<User> {
    fn get_user<T>(&self, id: T) -> Option<&User>
    where
        T: AsRef<Uuid>,
    {
        self.iter().find(|u| u.get_id() == id.as_ref())
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
