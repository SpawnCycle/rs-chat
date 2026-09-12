use crossterm::event::Event;
use ratatui::{
    Frame,
    layout::Rect,
    style::Stylize,
    text::Text,
    widgets::{Paragraph, Wrap},
};
use ratatui_textarea::{Input, Key};

use crate::{
    components::{AppContext, Component, EventResult},
    notifications::notification_to_span,
};

#[derive(Debug)]
pub struct NotificationView {
    scroll: u16,
}

impl NotificationView {
    #[must_use]
    pub fn new() -> Self {
        Self { scroll: 0 }
    }
}

impl Default for NotificationView {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for NotificationView {
    fn handle_event(&mut self, event: &Event, _ctx: &mut AppContext) -> EventResult {
        match event.clone().into() {
            Input { key: Key::Esc, .. } => {
                return EventResult::pop_component();
            }
            Input {
                key: Key::Char('d'),
                ctrl: true,
                ..
            }
            | Input {
                key: Key::PageDown, ..
            } => {
                self.scroll = self.scroll.saturating_add(1);
            }
            Input {
                key: Key::Char('u'),
                ctrl: true,
                ..
            }
            | Input {
                key: Key::PageUp, ..
            } => {
                self.scroll = self.scroll.saturating_sub(1);
            }
            _ => {}
        }

        EventResult::consumed()
    }

    fn render(&self, f: &mut Frame<'_>, area: Rect, ctx: &AppContext) {
        let para = if ctx.notifications().is_empty() {
            Paragraph::new("Empty".dark_gray()).centered()
        } else {
            let lines = ctx
                .notifications()
                .iter()
                .map(notification_to_span)
                .collect::<Text>();

            Paragraph::new(lines)
        }
        .wrap(Wrap { trim: false })
        .scroll((self.scroll, 0));

        f.render_widget(para, area);
    }
}
