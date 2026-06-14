mod context;
mod log_view;
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

impl EventResult {
    #[must_use]
    pub fn push_screen(screen: impl Component + 'static) -> Self {
        Self::Consumed(Some(AppAction::PushScreen(Box::new(screen))))
    }

    #[must_use]
    pub fn pop_screen() -> Self {
        Self::Consumed(Some(AppAction::PopScreen))
    }

    #[must_use]
    pub fn push_component(screen: impl Component + 'static) -> Self {
        Self::Consumed(Some(AppAction::PushComponent(Box::new(screen))))
    }

    #[must_use]
    pub fn pop_component() -> Self {
        Self::Consumed(Some(AppAction::PopComponent))
    }

    #[must_use]
    pub fn quit() -> Self {
        Self::Consumed(Some(AppAction::Quit))
    }

    #[must_use]
    pub fn consumed() -> Self {
        Self::Consumed(None)
    }

    #[must_use]
    pub fn ignored() -> Self {
        Self::Ignored
    }
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
