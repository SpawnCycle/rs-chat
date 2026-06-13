use crossterm::event::Event;
use ratatui::Frame;

use crate::{
    components::{AppAction, AppContext, Component, EventResult, RootComponent},
    config::AppConfig,
};

#[derive(Debug)]
pub struct App {
    screen_stack: Vec<Vec<Box<dyn Component>>>,
    context: AppContext,
    should_quit: bool,
}

#[allow(unused)]
impl App {
    #[must_use]
    pub fn new(config: AppConfig) -> Self {
        Self {
            screen_stack: vec![vec![Box::new(RootComponent::new())]],
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
        let screen = self.current_screen();
        let context = &self.context;

        for component in screen {
            component.render(f, f.area(), context);
        }
    }

    pub fn update(&mut self) {
        let context = &mut self.context;

        for screen in &mut self.screen_stack {
            for component in screen {
                component.update(context);
            }
        }
    }

    /// # Panics
    ///
    /// This function panics if there isn't a screen in the screen stack
    pub fn handle_event(&mut self, event: &Event) {
        let Self {
            context,
            screen_stack,
            ..
        } = self;
        let screen = screen_stack
            .last_mut()
            .expect("The stack should have at least 1 element");
        let mut pending_actions = Vec::new();

        for component in screen.iter_mut().rev() {
            let res = component.handle_event(event, context);

            if let EventResult::Consumed(Some(action)) = res {
                pending_actions.push(action);
            }
        }

        self.process_actions(&pending_actions);
    }

    pub fn quit(&mut self) {
        let context = &mut self.context;

        self.screen_stack.iter_mut().rev().for_each(|s| {
            for c in s.iter_mut() {
                c.before_quit(context);
            }
        });
    }

    #[must_use]
    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    fn process_actions(&mut self, actions: &[AppAction]) {
        for action in actions {
            match action {
                AppAction::PushScreen(component) => todo!(),
                AppAction::PopScreen => self.pop_screen(),
                AppAction::PushComponent(component) => todo!(),
                AppAction::PopComponent => self.pop_component(),
                AppAction::Quit => self.should_quit = true,
            }
        }
    }

    fn push_component(&mut self, component: impl Component + 'static) {
        self.current_screen_mut().push(Box::new(component));
    }

    fn pop_component(&mut self) {
        let screen = self.current_screen_mut();

        if screen.len() > 1 {
            screen.pop();
        } else {
            self.pop_screen();
        }
    }

    fn push_screen(&mut self, component: impl Component + 'static) {
        self.screen_stack.push(vec![Box::new(component)]);
    }

    fn pop_screen(&mut self) {
        if self.screen_stack.len() > 1 {
            self.screen_stack.pop();
        } else {
            self.should_quit = true;
        }
    }

    fn add_component(&mut self, component: impl Component + 'static) {
        self.current_screen_mut().push(Box::new(component));
    }

    fn current_screen(&self) -> &Vec<Box<dyn Component>> {
        self.screen_stack
            .last()
            .expect("The screen stack should have at least 1 screen")
    }

    fn current_screen_mut(&mut self) -> &mut Vec<Box<dyn Component>> {
        self.screen_stack
            .last_mut()
            .expect("The screen stack should have at least 1 screen")
    }
}
