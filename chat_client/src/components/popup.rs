use crossterm::event::Event;
use ratatui::{Frame, layout::Rect, widgets::Block};
use ratatui_textarea::{Input, Key};

use crate::components::{
    AppContext, BoxedComponent, Component, EventResult, popup_options::PopupOptions,
};

pub struct Popup {
    inner: BoxedComponent,
    options: PopupOptions,
}

impl std::fmt::Debug for Popup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Popup")
            .field("inner", &"<BoxedComponent>")
            .field("options", &self.options)
            .finish()
    }
}

impl Popup {
    #[must_use]
    pub fn new(component: BoxedComponent, options: PopupOptions) -> Self {
        Self {
            inner: component,
            options,
        }
    }
}

impl Component for Popup {
    fn handle_event(&mut self, event: &Event, ctx: &mut AppContext) -> EventResult {
        if let Input { key: Key::Esc, .. } = event.clone().into() {
            return EventResult::pop_component();
        }

        self.inner.handle_event(event, ctx)
    }

    fn render(&self, f: &mut Frame<'_>, area: Rect, ctx: &AppContext) {
        let area = area.centered(self.options.hsize, self.options.vsize);
        let mut block = Block::bordered();
        if let Some(name) = &self.options.name {
            block = block.title(name.as_str());
        }

        f.render_widget(&block, area);
        let area = block.inner(area);

        self.inner.render(f, area, ctx);
    }

    fn update(&mut self, ctx: &mut AppContext) {
        self.inner.update(ctx);
    }

    fn before_quit(&mut self, ctx: &mut AppContext) {
        self.inner.before_quit(ctx);
    }
}
