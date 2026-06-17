use std::fmt::{self, Debug};

use crossterm::event::Event;
use ratatui::{
    layout::Constraint,
    text::Line,
    widgets::{Block, Clear, Paragraph, Wrap},
};
use ratatui_textarea::{Input, Key};

use crate::components::{AppContext, Component, EventResult};

pub struct Popup {
    content: String,
    extra_toggle: Box<dyn Fn(&Event) -> bool>,
    /// scroll compared to the top
    scroll: u16,
    options: PopupOptions,
}

impl Debug for Popup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PopupComponent")
            .field("content", &self.content)
            .field("extra_toggle", &"<ExtraToggle>")
            .field("scroll", &self.scroll)
            .field("options", &self.options)
            .finish()
    }
}

#[derive(Debug)]
pub struct PopupOptions {
    /// toggles if the input should be passed through
    pass_input: bool,
    /// toggles if the user can use Ctrl+q to quit the whole app
    allow_quit: bool,
}

#[allow(dead_code)]
impl PopupOptions {
    pub fn new() -> Self {
        Self {
            pass_input: true,
            allow_quit: true,
        }
    }

    pub fn no_pass(self) -> Self {
        Self {
            pass_input: false,
            ..self
        }
    }

    pub fn pass(self) -> Self {
        Self {
            pass_input: true,
            ..self
        }
    }

    pub fn no_quit(self) -> Self {
        Self {
            allow_quit: false,
            ..self
        }
    }

    pub fn quit(self) -> Self {
        Self {
            allow_quit: true,
            ..self
        }
    }
}

impl Popup {
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

impl Component for Popup {
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
        let area = area.centered(Constraint::Percentage(75), Constraint::Percentage(75));
        let lines = self.content.lines().map(Line::from).collect::<Vec<_>>();

        let para = Paragraph::new(lines)
            .block(Block::bordered())
            .wrap(Wrap { trim: false })
            .scroll((self.scroll, 0));

        // clear the area, because some of the text will be left over during the switch
        f.render_widget(Clear, area);
        f.render_widget(para, area);
    }

    fn update(&self, _ctx: &mut super::AppContext) {}

    fn before_quit(&mut self, _ctx: &mut super::AppContext) {}
}
