use std::{collections::HashSet, str::FromStr};

use crossterm::event::Event;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph},
};
use ratatui_textarea::{CursorMove, Input, Key, TextArea};
use url::Url;

use crate::{
    components::{AppContext, Component, EventResult},
    consts::{FOCUSED_CURSOR_STYLE, UNFOCUSED_CURSOR_STYLE},
    helper::{RoomLocation, ServerUrl, apply_cursor_style, text_area},
};

// TODO: implement the ability to select from which server's rooms you want to search
#[derive(Debug)]
pub struct RoomSwitchModal<'a> {
    /// The server url whose rooms are searched
    search_url: Url,
    server_field: TextArea<'a>,
    room_field: TextArea<'a>,
    searching_rooms: bool,

    /// Context dependent, can be either the matched rooms or servers
    matched_rows: Vec<String>,
}

impl RoomSwitchModal<'_> {
    #[must_use]
    pub fn new(search_url: Url) -> Self {
        let url_block = Block::bordered().title("Server url");
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
            server_field,
            room_field,
            search_url,
            searching_rooms: true,
            matched_rows: Vec::new(),
        }
    }

    /// TODO: sort based on matched %?
    fn update_matches(&mut self, ctx: &mut AppContext) {
        if self.searching_rooms {
            let query = self.room_field.lines()[0].clone();

            let names = ctx
                .rooms
                .iter()
                .filter(|r| r.0.url == self.search_url)
                .map(|r| r.0.room_name.clone())
                .filter(|r| r.contains(&query))
                .collect::<HashSet<_>>();

            self.matched_rows = names.into_iter().collect();
        } else {
            let servers = ctx
                .rooms
                .iter()
                .filter(|r| r.0.url == self.search_url)
                .map(|r| r.0.url.to_string())
                .collect::<HashSet<_>>();

            self.matched_rows = servers.into_iter().collect();
        }

        self.matched_rows.sort();
    }

    /// TODO: I'm sure this can be done better
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

        if names.len() == 1 {
            let loc = RoomLocation::new(self.search_url.clone(), names[0].clone());
            ctx.try_set_current_room(loc);
        }

        EventResult::pop_component()
    }

    fn toggle_search_type(&mut self, ctx: &mut AppContext) {
        if !self.searching_rooms {
            let server = self.server_field.lines()[0].clone();
            let server = ServerUrl::from_str(&server);
            if let Ok(ServerUrl(url)) = server
                && ctx.has_server(&url)
            {
                self.search_url = url;
            }
            self.server_field.clear();
            self.server_field.insert_str(&self.search_url);
        }

        self.searching_rooms = !self.searching_rooms;

        apply_cursor_style(
            &mut self.room_field,
            &mut self.server_field,
            self.searching_rooms,
        );

        self.update_matches(ctx);
    }
}

impl Component for RoomSwitchModal<'_> {
    fn handle_event(&mut self, event: &Event, ctx: &mut AppContext) -> EventResult {
        match event.clone().into() {
            Input { key: Key::Tab, .. } => {
                self.toggle_search_type(ctx);
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
                if self.searching_rooms {
                    self.room_field.input(event.clone());
                } else {
                    self.server_field.input(event.clone());
                }

                self.update_matches(ctx);
            }
        }

        EventResult::consumed()
    }

    fn render(&self, f: &mut Frame<'_>, area: Rect, _ctx: &AppContext) {
        let layout = Layout::new(
            Direction::Vertical,
            [
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Fill(1),
            ],
        );
        let area = layout.split(area);

        let para = Paragraph::new(
            self.matched_rows
                .iter()
                .map(|l| Line::from(l.as_str()))
                .collect::<Text>(),
        )
        .block(Block::new().borders(Borders::TOP).title("Matches"));

        f.render_widget(&self.server_field, area[0]);
        f.render_widget(&self.room_field, area[1]);
        f.render_widget(&para, area[2]);
    }

    fn update(&mut self, ctx: &mut AppContext) {
        self.update_matches(ctx);
    }
}
