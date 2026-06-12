use crossterm::event::Event;
use ratatui::Frame;

use crate::{
    components::{AppAction, AppContext, Component, EventResult, RootComponent},
    config::AppConfig,
};

#[derive(Debug)]
pub struct App {
    render_stack: Vec<Box<dyn Component>>,
    context: AppContext,
    should_quit: bool,
}

#[allow(unused)]
impl App {
    #[must_use]
    pub fn new(config: AppConfig) -> Self {
        Self {
            render_stack: vec![Box::new(RootComponent::new())],
            context: AppContext::new(config),
            should_quit: false,
        }
    }

    /// This functions mocks some of the unimplemented features
    ///
    /// # Errors
    ///
    /// This function errors if there was a problem during the setup of various mock features,
    /// usually it's a network error
    pub async fn mock_unimplemented(&mut self) -> anyhow::Result<()> {
        self.context.join_room("default").await?;

        Ok(())
    }

    pub fn render(&self, f: &mut Frame<'_>) {
        for component in &self.render_stack {
            component.render(f, f.area(), &self.context);
        }
    }

    pub fn update(&mut self) {
        for component in &mut self.render_stack {
            component.update(&mut self.context);
        }
    }

    pub fn handle_event(&mut self, event: &Event) {
        for component in self.render_stack.iter_mut().rev() {
            let res = component.handle_event(event, &mut self.context);

            if let EventResult::Consumed(Some(AppAction::Quit)) = res {
                self.should_quit = true;
            }
        }
    }

    pub fn quit(&mut self) {
        self.render_stack
            .iter_mut()
            .rev()
            .for_each(|c| c.before_quit(&mut self.context));
    }

    #[must_use]
    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    fn add_component(&mut self, component: impl Component + 'static) {
        self.render_stack.push(Box::new(component));
    }
}
