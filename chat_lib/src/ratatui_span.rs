use crate::types::User;
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

impl AsSpan for User {
    fn as_span(&self) -> Span<'_> {
        Span::from(self.get_name())
    }
}

impl FindUser for Vec<User> {
    fn get_user<T>(&self, id: T) -> Option<&User>
    where
        T: AsRef<Uuid>,
    {
        self.iter().find(|u| u.get_id() == id.as_ref())
    }
}
