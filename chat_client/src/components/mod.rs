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
    /// Adds a new screen and switches to it
    PushScreen(Box<dyn Component>),
    /// Removes the last screen from the stack,
    /// quits if the current one was the last one
    PopScreen,
    /// Adds a new component to the render stack,
    /// good for popups and modals
    PushComponent(Box<dyn Component>),
    /// Removes the last component from the stack,
    /// quits the screen if the current component was the last one
    PopComponent,
    Quit,
}
