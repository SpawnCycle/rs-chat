use crossterm::event::Event;
use ratatui::{Frame, layout::Rect};
use ratatui_textarea::{Input, Key};

use crate::components::{AppContext, BoxedComponent, Component, EventResult};

pub struct Screen {
    inner: BoxedComponent,
}

impl std::fmt::Debug for Screen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Screen")
            .field("inner", &"<BoxedComponent>")
            .finish()
    }
}

impl Screen {
    pub fn new(component: impl Component + 'static) -> Self {
        Self {
            inner: component.boxed(),
        }
    }
}

impl Component for Screen {
    fn handle_event(&mut self, event: &Event, ctx: &mut AppContext) -> EventResult {
        match event.clone().into() {
            Input {
                key: Key::Char('q'),
                ctrl: true,
                alt: false,
                shift: false,
            } => EventResult::pop_screen(),
            _ => self.inner.handle_event(event, ctx),
        }
    }

    fn render(&self, f: &mut Frame<'_>, area: Rect, ctx: &AppContext) {
        self.inner.render(f, area, ctx);
    }

    fn update(&mut self, ctx: &mut AppContext) {
        self.inner.update(ctx);
    }

    fn before_quit(&mut self, ctx: &mut AppContext) {
        self.inner.before_quit(ctx);
    }
}
