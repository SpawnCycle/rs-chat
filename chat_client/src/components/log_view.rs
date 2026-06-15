use crossterm::event::Event;
use ratatui::{Frame, layout::Rect};
use ratatui_textarea::{Input, Key};
use tui_logger::TuiWidgetEvent;

use crate::{
    components::{AppAction, AppContext, Component, EventResult},
    logs::draw_logs,
};

pub struct LogViewComponent {
    logger_state: tui_logger::TuiWidgetState,
}

impl std::fmt::Debug for LogViewComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LogViewComponent")
            .field("logger_state", &"<LoggerState>")
            .finish()
    }
}

impl Component for LogViewComponent {
    fn handle_event(&mut self, event: &Event, _ctx: &mut AppContext) -> EventResult {
        self.handle_log_input(event.clone())
    }

    fn render(&self, f: &mut Frame<'_>, area: Rect, _ctx: &AppContext) {
        draw_logs(f, area, &self.logger_state);
    }

    fn update(&self, _ctx: &mut AppContext) {}

    fn before_quit(&mut self, _ctx: &mut AppContext) {}
}

impl LogViewComponent {
    pub fn new() -> Self {
        Self {
            logger_state: tui_logger::TuiWidgetState::new(),
        }
    }

    fn handle_log_input(&mut self, e: Event) -> EventResult {
        match e.into() {
            Input {
                key: Key::Char('l'),
                ctrl: true,
                ..
            } => return EventResult::Consumed(Some(AppAction::PopupComponent)),
            Input {
                key: Key::Char(' '),
                ..
            } => {
                self.logger_state.transition(TuiWidgetEvent::SpaceKey);
            }
            Input { key: Key::Esc, .. } => {
                self.logger_state.transition(TuiWidgetEvent::EscapeKey);
            }
            Input {
                key: Key::PageUp, ..
            } => {
                self.logger_state.transition(TuiWidgetEvent::PrevPageKey);
            }
            Input {
                key: Key::PageDown, ..
            } => {
                self.logger_state.transition(TuiWidgetEvent::NextPageKey);
            }
            Input { key: Key::Up, .. } => {
                self.logger_state.transition(TuiWidgetEvent::UpKey);
            }
            Input { key: Key::Down, .. } => {
                self.logger_state.transition(TuiWidgetEvent::DownKey);
            }
            Input { key: Key::Left, .. } => {
                self.logger_state.transition(TuiWidgetEvent::LeftKey);
            }
            Input {
                key: Key::Right, ..
            } => {
                self.logger_state.transition(TuiWidgetEvent::RightKey);
            }
            Input {
                key: Key::Char('+'),
                ..
            } => {
                self.logger_state.transition(TuiWidgetEvent::PlusKey);
            }
            Input {
                key: Key::Char('-'),
                ..
            } => {
                self.logger_state.transition(TuiWidgetEvent::MinusKey);
            }
            Input {
                key: Key::Char('h'),
                ..
            } => {
                self.logger_state.transition(TuiWidgetEvent::HideKey);
            }
            Input {
                key: Key::Char('f'),
                ..
            } => {
                self.logger_state.transition(TuiWidgetEvent::FocusKey);
            }
            Input {
                key: Key::Char('q'),
                ctrl: true,
                ..
            } => {
                return EventResult::quit();
            }
            _ => {
                // TODO: some other controls?
                return EventResult::ignored();
            }
        }

        EventResult::consumed()
    }
}
