use crossterm::event::Event;
use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    widgets::Block,
};
use ratatui_textarea::{Input, Key, TextArea};

use crate::components::{AppAction, Component, EventResult};

#[derive(Debug)]
pub struct RoomJoinComponent<'a> {
    message_field: TextArea<'a>,
}

impl RoomJoinComponent<'_> {
    pub fn new() -> Self {
        Self {
            message_field: TextArea::new(Vec::new()),
        }
    }
}

impl Component for RoomJoinComponent<'_> {
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
                    AppAction::join_room(input),
                    AppAction::pop_component(),
                ]);
            }
            Input { key: Key::Esc, .. } => {
                return EventResult::pop_component();
            }
            _ => {
                self.message_field.input(event.clone());
            }
        }

        EventResult::consumed()
    }

    fn render(&self, f: &mut Frame<'_>, area: Rect, _ctx: &super::AppContext) {
        let block = Block::bordered().title_top("Join a room");
        let area = area.centered(Constraint::Percentage(75), Constraint::Length(3));

        f.render_widget(&block, area);
        let area = block.inner(area);

        f.render_widget(&self.message_field, area);
    }

    fn update(&self, _ctx: &mut super::AppContext) {}

    fn before_quit(&mut self, _ctx: &mut super::AppContext) {}
}
