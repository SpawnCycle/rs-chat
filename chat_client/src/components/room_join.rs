use std::str::FromStr;

use crossterm::event::Event;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    widgets::Block,
};
use ratatui_textarea::{CursorMove, Input, Key, TextArea};
use url::Url;

use crate::{
    components::{AppAction, Component, EventResult},
    helper::{ServerUrl, apply_cursor_style, text_area},
};

/// TODO: make this better
#[derive(Debug)]
pub struct RoomJoinModal<'a> {
    // TODO: don't do this, make a new text field instead
    server_url: Url,
    message_field: TextArea<'a>,
    url_field: TextArea<'a>,
    typing_url: bool,
}

impl RoomJoinModal<'_> {
    pub fn new(server_url: Url) -> Self {
        let url_block = Block::bordered().title("Server url");
        let message_block = Block::bordered().title("Room name");

        let mut url_field = text_area();
        url_field.insert_str(&server_url);
        url_field.set_block(url_block);
        url_field.set_cursor_line_style(Style::new().not_underlined());
        // For some reason it doesn't move it by itself
        url_field.move_cursor(CursorMove::End);

        let mut message_field = text_area();
        message_field.set_block(message_block);
        message_field.set_cursor_line_style(Style::new().not_underlined());

        apply_cursor_style(&mut url_field, &mut message_field, false);

        Self {
            server_url,
            message_field,
            url_field,
            typing_url: false,
        }
    }

    fn apply_cursor_style(&mut self) {
        apply_cursor_style(
            &mut self.url_field,
            &mut self.message_field,
            self.typing_url,
        );
    }

    fn switch_inputs(&mut self) {
        if self.typing_url {
            let url_text = self.url_field.lines()[0].clone();
            let url = ServerUrl::from_str(url_text.trim());
            if let Ok(url) = url {
                self.server_url = url.into();
            }
            self.url_field.clear();
            self.url_field.insert_str(&self.server_url);
        }

        self.typing_url = !self.typing_url;
        self.apply_cursor_style();
    }
}

impl Component for RoomJoinModal<'_> {
    fn handle_event(&mut self, event: &Event, _ctx: &mut super::AppContext) -> super::EventResult {
        match event.clone().into() {
            Input {
                key: Key::Char('m'),
                ctrl: true,
                ..
            }
            | Input {
                key: Key::Enter, ..
            } => {
                let input = self.message_field.lines()[0].trim();
                return EventResult::batch([
                    AppAction::join_room(self.server_url.clone(), input),
                    AppAction::pop_component(),
                ]);
            }
            Input { key: Key::Tab, .. } => {
                self.switch_inputs();
            }
            _ => {
                if self.typing_url {
                    self.url_field.input(event.clone());
                } else {
                    self.message_field.input(event.clone());
                }
            }
        }

        EventResult::consumed()
    }

    fn render(&self, f: &mut Frame<'_>, area: Rect, _ctx: &super::AppContext) {
        let layout = Layout::new(
            Direction::Vertical,
            [Constraint::Length(3), Constraint::Length(3)],
        );
        let area = layout.split(area);

        f.render_widget(&self.url_field, area[0]);
        f.render_widget(&self.message_field, area[1]);
    }
}
