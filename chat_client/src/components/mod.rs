mod context;
mod log_view;
mod popup;
mod room_join;
mod room_switch;
mod root;
mod text_popup;

pub mod popup_options;

use std::fmt::Debug;

use crossterm::event::Event;
use ratatui::{Frame, layout::Rect};
use url::Url;

pub use context::AppContext;
pub use root::Root;

pub type BoxedComponent = Box<dyn Component>;

pub trait Component {
    // functions that should be implemented

    fn handle_event(&mut self, event: &Event, ctx: &mut AppContext) -> EventResult;
    fn render(&self, f: &mut Frame<'_>, area: Rect, ctx: &AppContext);

    // functions that can be implemented

    fn update(&self, _ctx: &mut AppContext) {}
    fn before_quit(&mut self, _ctx: &mut AppContext) {}

    // functions that shouldn't be overridden

    fn boxed(self) -> BoxedComponent
    where
        Self: Sized + 'static,
    {
        Box::new(self)
    }
}

#[derive(Debug)]
pub enum EventResult {
    Consumed(Vec<AppAction>),
    Ignored,
}

impl EventResult {
    pub fn batch(actions: impl IntoIterator<Item = AppAction>) -> Self {
        Self::Consumed(actions.into_iter().collect())
    }

    #[allow(clippy::needless_pass_by_value)]
    #[must_use]
    pub fn join_room(url: impl Into<Url>, name: impl ToString) -> Self {
        Self::batch([AppAction::join_room(url, name)])
    }

    #[must_use]
    pub fn push_screen(screen: impl Component + 'static) -> Self {
        Self::batch([AppAction::push_screen(screen)])
    }

    #[must_use]
    pub fn pop_screen() -> Self {
        Self::batch([AppAction::pop_screen()])
    }

    #[must_use]
    pub fn push_component(component: impl Component + 'static) -> Self {
        Self::batch([AppAction::push_component(component)])
    }

    #[must_use]
    pub fn pop_component() -> Self {
        Self::batch([AppAction::pop_component()])
    }

    #[must_use]
    pub fn quit() -> Self {
        Self::batch([AppAction::quit()])
    }

    #[must_use]
    pub fn consumed() -> Self {
        Self::batch([])
    }

    #[must_use]
    pub fn ignored() -> Self {
        Self::Ignored
    }
}

pub enum AppAction {
    /// Adds a new screen and switches to it
    PushScreen(BoxedComponent),
    /// Removes the last screen from the stack,
    /// quits if the current one was the last one
    PopScreen,
    /// Adds a new component to the render stack,
    /// good for popups and modals
    PushComponent(BoxedComponent),
    /// Removes the last component from the stack,
    /// quits the screen if the current component was the last one
    PopComponent,
    /// Tries to join the room with the given name
    JoinRoom(Url, String),
    Quit,
}

impl Debug for AppAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PushScreen(_) => f.debug_tuple("<PushScreen>").finish(),
            Self::PopScreen => write!(f, "PopScreen"),
            Self::PushComponent(_) => f.debug_tuple("<PushComponent>").finish(),
            Self::PopComponent => write!(f, "PopComponent"),
            Self::JoinRoom(arg0, arg1) => {
                f.debug_tuple("JoinRoom").field(arg0).field(arg1).finish()
            }
            Self::Quit => write!(f, "Quit"),
        }
    }
}

impl AppAction {
    pub fn batch(actions: impl IntoIterator<Item = AppAction>) -> Vec<Self> {
        actions.into_iter().collect()
    }

    #[allow(clippy::needless_pass_by_value)]
    #[must_use]
    pub fn join_room(url: impl Into<Url>, name: impl ToString) -> Self {
        AppAction::JoinRoom(url.into(), name.to_string())
    }

    #[must_use]
    pub fn push_screen(screen: impl Component + 'static) -> Self {
        AppAction::PushScreen(Box::new(screen))
    }

    #[must_use]
    pub fn pop_screen() -> Self {
        AppAction::PopScreen
    }

    #[must_use]
    pub fn push_component(screen: impl Component + 'static) -> Self {
        AppAction::PushComponent(Box::new(screen))
    }

    #[must_use]
    pub fn pop_component() -> Self {
        AppAction::PopComponent
    }

    #[must_use]
    pub fn quit() -> Self {
        AppAction::Quit
    }
}
