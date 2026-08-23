use crossterm::event::Event;
use ratatui::layout::Constraint;

pub type ExitToggleFn = Box<dyn Fn(&Event) -> bool>;

pub struct PopupOptions {
    /// sets the string
    pub name: Option<String>,
    /// toggles if the input should be passed through
    pub pass_input: bool,
    /// toggles if the user can use Ctrl+q to quit the whole app
    pub allow_quit: bool,
    /// sets the size of the popup
    pub hsize: Constraint,
    pub vsize: Constraint,

    /// binds with which the popup can be exited
    pub exit_binds: Vec<ExitToggleFn>,
}

impl std::fmt::Debug for PopupOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PopupOptions")
            .field("name", &self.name)
            .field("pass_input", &self.pass_input)
            .field("allow_quit", &self.allow_quit)
            .field("hsize", &self.hsize)
            .field("vsize", &self.vsize)
            .field("exit_binds", &"<ExtraToggles>")
            .finish()
    }
}

impl Default for PopupOptions {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl PopupOptions {
    #[must_use]
    pub fn new() -> Self {
        Self {
            name: None,
            pass_input: true,
            allow_quit: true,
            hsize: Constraint::Percentage(75),
            vsize: Constraint::Percentage(75),
            exit_binds: Vec::new(),
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    #[must_use]
    pub fn set_name(self, name: impl ToString) -> Self {
        Self {
            name: Some(name.to_string()),
            ..self
        }
    }

    #[must_use]
    pub fn no_name(self) -> Self {
        Self { name: None, ..self }
    }

    #[must_use]
    pub fn set_hsize(self, size: Constraint) -> Self {
        Self {
            hsize: size,
            ..self
        }
    }

    #[must_use]
    pub fn set_vsize(self, size: Constraint) -> Self {
        Self {
            vsize: size,
            ..self
        }
    }

    #[must_use]
    pub fn no_pass(self) -> Self {
        Self {
            pass_input: false,
            ..self
        }
    }

    #[must_use]
    pub fn pass(self) -> Self {
        Self {
            pass_input: true,
            ..self
        }
    }

    #[must_use]
    pub fn no_quit(self) -> Self {
        Self {
            allow_quit: false,
            ..self
        }
    }

    #[must_use]
    pub fn quit(self) -> Self {
        Self {
            allow_quit: true,
            ..self
        }
    }

    #[must_use]
    pub fn empty_exit(mut self) -> Self {
        self.exit_binds.clear();

        self
    }

    #[must_use]
    pub fn add_exit(mut self, bind: impl Fn(&Event) -> bool + 'static) -> Self {
        self.exit_binds.push(Box::new(bind));

        self
    }
}
