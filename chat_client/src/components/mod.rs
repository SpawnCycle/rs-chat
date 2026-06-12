mod context;
mod root;

use std::fmt::Debug;

use crossterm::event::Event;
use ratatui::{Frame, layout::Rect};

pub use context::AppContext;
pub use root::RootComponent;

pub trait Component: Debug {
    fn handle_event(&mut self, event: &Event, ctx: &mut AppContext) -> EventResult;
    fn render(&self, f: &mut Frame<'_>, area: Rect, ctx: &AppContext);
    fn update(&self, ctx: &mut AppContext);

    fn before_quit(&mut self, ctx: &mut AppContext);
}

#[derive(Debug)]
pub enum EventResult {
    Consumed(Option<AppAction>),
    Ignored,
}

#[derive(Debug)]
pub enum AppAction {
    Quit,
}
