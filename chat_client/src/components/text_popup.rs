use std::fmt::{self, Debug};

use crossterm::event::Event;
use ratatui::{
    text::Line,
    widgets::{Block, Clear, Paragraph, Wrap},
};
use ratatui_textarea::{Input, Key};

use crate::components::{AppContext, Component, EventResult, popup_options::PopupOptions};

pub struct TextPopup {
    content: String,
    extra_toggle: Box<dyn Fn(&Event) -> bool>,
    /// scroll compared to the top
    scroll: u16,
    options: PopupOptions,
}

impl Debug for TextPopup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PopupComponent")
            .field("content", &self.content)
            .field("extra_toggle", &"<ExtraToggle>")
            .field("scroll", &self.scroll)
            .field("options", &self.options)
            .finish()
    }
}

impl TextPopup {
    pub fn new(
        content: &(impl ToString + ?Sized),
        options: PopupOptions,
        extra_toggle: impl Fn(&Event) -> bool + 'static,
    ) -> Self {
        Self {
            scroll: 0,
            content: content.to_string(),
            extra_toggle: Box::new(extra_toggle),
            options,
        }
    }
}

impl Component for TextPopup {
    fn handle_event(&mut self, event: &Event, _ctx: &mut AppContext) -> super::EventResult {
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
            Input {
                key: Key::Char('q'),
                ctrl: true,
                ..
            } if self.options.allow_quit => {
                return EventResult::quit();
            }
            _ => {
                if (self.extra_toggle)(event) {
                    return EventResult::pop_component();
                }

                return if self.options.pass_input {
                    EventResult::ignored()
                } else {
                    EventResult::consumed()
                };
            }
        }

        EventResult::consumed()
    }

    fn render(
        &self,
        f: &mut ratatui::prelude::Frame<'_>,
        area: ratatui::prelude::Rect,
        _ctx: &super::AppContext,
    ) {
        let area = area.centered(self.options.hsize, self.options.vsize);
        let lines = self.content.lines().map(Line::from).collect::<Vec<_>>();
        let mut block = Block::bordered();

        if let Some(name) = &self.options.name {
            block = block.title(name.as_str());
        }

        let para = Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false })
            .scroll((self.scroll, 0));

        // clear the area, because some of the text will be left over during the switch
        f.render_widget(Clear, area);
        f.render_widget(para, area);
    }
}
