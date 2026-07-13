use crossterm::event::Event;
use ratatui::{
    style::Stylize,
    text::{Line, ToSpan},
    widgets::{Clear, Paragraph},
};
use ratatui_textarea::{Input, Key};

use crate::components::{AppContext, Component, EventResult};

#[derive(Debug)]
pub struct UserView {
    scroll: u16,
}

impl UserView {
    pub const fn new() -> Self {
        Self { scroll: 0 }
    }
}

impl Default for UserView {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for UserView {
    fn handle_event(&mut self, event: &Event, _ctx: &mut AppContext) -> EventResult {
        match event.clone().into() {
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
            _ => {
                return EventResult::consumed();
            }
        }

        EventResult::consumed()
    }

    fn render(
        &self,
        f: &mut ratatui::prelude::Frame<'_>,
        area: ratatui::prelude::Rect,
        ctx: &super::AppContext,
    ) {
        let mut lines = Vec::new();
        if let Some(r) = ctx.current_room() {
            for (id, usr) in r.users() {
                lines.push(Line::from_iter([
                    usr.get_name().blue(),
                    " ".to_span(),
                    format!("({id})").dark_gray(),
                ]));
            }
            let para = Paragraph::new(lines).scroll((self.scroll, 0));

            f.render_widget(para, area);
        } else {
            f.render_widget(Clear, area);
        }
    }
}
