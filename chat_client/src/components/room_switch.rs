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
    components::{AppContext, Component, EventResult},
    consts::{FOCUSED_CURSOR_STYLE, UNFOCUSED_CURSOR_STYLE},
    helper::text_area,
};

// TODO: implement the ability to select from which server's rooms you want to search
pub struct RoomSwitchModal<'a> {
    /// The server url whose rooms are searched
    search_url: Url,
    server_field: TextArea<'a>,
    room_field: TextArea<'a>,
}

impl RoomSwitchModal<'_> {
    pub fn new(search_url: Url) -> Self {
        let url_block = Block::bordered().title("Server url (placeholder)");
        let message_block = Block::bordered().title("Room name");

        let mut server_field = text_area();
        server_field.insert_str(&search_url);
        server_field.set_block(url_block);
        server_field.set_cursor_line_style(Style::new().not_underlined());
        server_field.set_cursor_style(UNFOCUSED_CURSOR_STYLE);
        // For some reason it doesn't move it by itself
        server_field.move_cursor(CursorMove::End);

        let mut room_field = text_area();
        room_field.set_block(message_block);
        room_field.set_cursor_line_style(Style::new().not_underlined());
        room_field.set_cursor_style(FOCUSED_CURSOR_STYLE);

        Self {
            search_url,
            server_field,
            room_field,
        }
    }

    fn try_switch_room(&mut self, ctx: &mut AppContext) -> EventResult {
        let query = self.room_field.lines()[0].clone();
        self.room_field.clear();

        let names = ctx
            .rooms
            .iter()
            .filter(|r| r.0.url == self.search_url)
            .map(|r| r.0.room_name.clone())
            .filter(|r| r.contains(&query))
            .collect::<Vec<_>>();

        log::debug!("room matches: {names:?}");

        EventResult::pop_component()
    }
}

impl Component for RoomSwitchModal<'_> {
    fn handle_event(&mut self, event: &Event, ctx: &mut AppContext) -> EventResult {
        match event.clone().into() {
            Input { key: Key::Esc, .. } => {
                return EventResult::pop_component();
            }
            Input {
                key: Key::Char('m'),
                ctrl: true,
                ..
            }
            | Input {
                key: Key::Enter, ..
            } => {
                return self.try_switch_room(ctx);
            }
            _ => {
                self.room_field.input(event.clone());
            }
        }

        EventResult::consumed()
    }

    fn render(&self, f: &mut Frame<'_>, area: Rect, _ctx: &AppContext) {
        let area = area.centered(Constraint::Percentage(75), Constraint::Length(8));

        let top_block = Block::bordered().title("Switch to a room (placeholder)");
        f.render_widget(&top_block, area);
        let area = top_block.inner(area);

        let layout = Layout::new(
            Direction::Vertical,
            [Constraint::Length(3), Constraint::Length(3)],
        );
        let area = layout.split(area);

        f.render_widget(&self.server_field, area[0]);
        f.render_widget(&self.room_field, area[1]);
    }
}
