use crossterm::event::Event;
use ratatui::Frame;

use crate::{
    AppError, AppEvent,
    components::{AppAction, AppContext, Component, EventResult, RootComponent},
    config::AppConfig,
};

#[derive(Debug)]
pub struct App {
    screen_stack: Vec<Vec<Box<dyn Component>>>,
    context: AppContext,
    exit_reason: Option<ExitReason>,
}

#[derive(Debug, Default)]
pub enum ExitReason {
    #[default]
    UserAction,
    BackgroundError(AppError),
    FatalError(anyhow::Error),
}

impl From<AppError> for ExitReason {
    fn from(value: AppError) -> Self {
        Self::BackgroundError(value)
    }
}

impl From<anyhow::Error> for ExitReason {
    fn from(value: anyhow::Error) -> Self {
        Self::FatalError(value)
    }
}

#[allow(unused)]
impl App {
    #[must_use]
    pub fn new(config: AppConfig) -> Self {
        Self {
            screen_stack: vec![vec![Box::new(RootComponent::new())]],
            context: AppContext::new(config),
            exit_reason: None,
        }
    }

    /// This functions mocks some of the unimplemented features
    ///
    /// # Errors
    ///
    /// This function errors if there was a problem during the setup of various mock features,
    /// usually it's a network error
    pub async fn mock_unimplemented(&mut self) -> anyhow::Result<()> {
        let default_room = self.context.config.web.default_room.clone();
        self.context.join_room(&default_room).await?;

        Ok(())
    }

    pub fn render(&self, f: &mut Frame<'_>) {
        let screen = self.current_screen();
        let context = &self.context;

        for component in screen {
            component.render(f, f.area(), context);
        }
    }

    /// This function may be async if the event triggers an action that is async
    pub async fn handle_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::Tick => {
                self.update();
            }
            AppEvent::Event(event) => self.handle_input(&event).await,
            AppEvent::Error(err) => {
                log::error!("Background error: {err}");
                self.exit_reason = Some(ExitReason::from(err));
            }
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
    /// This function panics if there isn't a screen on the screen stack
    pub async fn handle_input(&mut self, event: &Event) {
        let Self {
            context,
            screen_stack,
            ..
        } = self;
        let screen = screen_stack
            .last_mut()
            .expect("The stack should have at least 1 element");

        for component in screen.iter_mut().rev() {
            let res = component.handle_event(event, context);

            if let EventResult::Consumed(res) = res {
                if let Some(action) = res {
                    self.process_action(action).await;
                }
                break;
            }
        }
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
        self.exit_reason.is_some()
    }

    /// Returns the reason for exiting the app
    ///
    /// Returns `None` if called before the app would exit
    #[must_use]
    pub fn exit_reason(&self) -> Option<&ExitReason> {
        self.exit_reason.as_ref()
    }

    pub fn exit_because(&mut self, err: anyhow::Error) {
        self.exit_reason = Some(ExitReason::from(err));
    }

    async fn process_actions(&mut self, actions: Vec<AppAction>) {
        for action in actions {
            self.process_action(action).await;
        }
    }

    async fn process_action(&mut self, action: AppAction) {
        match action {
            AppAction::Batch(actions) => {
                // The `Box::pin` is needed because this is a possibly infinite recusive call
                Box::pin(self.process_actions(actions)).await;
            }
            AppAction::PushScreen(screen) => self.push_screen(screen),
            AppAction::PopScreen => self.pop_screen(),
            AppAction::PushComponent(component) => self.push_component(component),
            AppAction::PopComponent => self.pop_component(),
            AppAction::JoinRoom(name) => {
                self.context.join_room(&name).await;
            }
            AppAction::Quit => self.exit_reason = Some(ExitReason::default()),
        }
    }

    fn push_component(&mut self, component: Box<dyn Component>) {
        self.current_screen_mut().push(component);
    }

    fn pop_component(&mut self) {
        let Self {
            context,
            screen_stack,
            ..
        } = self;
        let screen = screen_stack
            .last_mut()
            .expect("The stack should have at least 1 element");

        if screen.len() > 1 {
            if let Some(mut component) = screen.pop() {
                component.before_quit(context);
            }
        } else {
            self.pop_screen();
        }
    }

    fn push_screen(&mut self, component: Box<dyn Component>) {
        self.screen_stack.push(vec![component]);
    }

    fn pop_screen(&mut self) {
        if self.screen_stack.len() > 1 {
            if let Some(screen) = self.screen_stack.pop() {
                for mut component in screen.into_iter().rev() {
                    component.before_quit(&mut self.context);
                }
            }
        } else {
            self.exit_reason = Some(ExitReason::default());
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
