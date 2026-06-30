use crossterm::event::Event;
use ratatui::{
    Frame,
    layout::Rect,
    text::Text,
    widgets::{Paragraph, Wrap},
};
use ratatui_textarea::{Input, Key};

use crate::{
    components::{AppContext, Component, EventResult},
    notifications::{self, Notification, notification_to_line},
};

#[derive(Debug)]
pub struct NotificationView {
    notifications: Vec<Notification>,
    // distance from the last notification
    scroll: u16,
}

impl NotificationView {
    #[must_use]
    pub fn new() -> Self {
        Self {
            notifications: Vec::new(),
            scroll: 0,
        }
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
                self.scroll = self.scroll.saturating_sub(1);
            }
            Input {
                key: Key::Char('u'),
                ctrl: true,
                ..
            }
            | Input {
                key: Key::PageUp, ..
            } => {
                self.scroll = self.scroll.saturating_add(1);
            }
            _ => {}
        }

        EventResult::consumed()
    }

    fn render(&self, f: &mut Frame<'_>, area: Rect, _ctx: &AppContext) {
        let lines = self
            .notifications
            .iter()
            .map(notification_to_line)
            .collect::<Text>();

        #[allow(clippy::cast_possible_truncation)]
        let scroll = (lines.height() as u16).saturating_sub(self.scroll);

        let para = Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .scroll((scroll, 0));

        f.render_widget(para, area);
    }

    fn update(&mut self, _ctx: &mut AppContext) {
        if let Some(notifs) = notifications::try_get_notifications() {
            self.notifications = notifs;
        }
    }
}
