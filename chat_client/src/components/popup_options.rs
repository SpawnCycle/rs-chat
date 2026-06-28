use ratatui::layout::Constraint;

#[derive(Debug, Clone)]
pub struct PopupOptions {
    // sets the string
    pub name: Option<String>,
    /// toggles if the input should be passed through
    pub pass_input: bool,
    /// toggles if the user can use Ctrl+q to quit the whole app
    pub allow_quit: bool,
    /// sets the size of the popup
    pub hsize: Constraint,
    pub vsize: Constraint,
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
}
