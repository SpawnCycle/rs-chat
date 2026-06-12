use std::fmt::{self, Debug, Formatter};

use crossterm::event::Event;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect, Spacing},
    symbols::merge::MergeStrategy,
    text::Text,
    widgets::{Block, Borders, Clear, Paragraph},
};
use ratatui_textarea::{Input, Key, TextArea};
use tui_logger::TuiWidgetEvent;

use crate::{
    chat::{draw_room_events, draw_top_bar, top_block},
    components::{AppAction, AppContext, Component, EventResult},
    logs::draw_logs,
};

// TODO: seperate the logger into a different component
pub struct RootComponent<'a> {
    message_field: TextArea<'a>,
    active_text_area: Option<ActiveTextArea<'a>>,
    checking_logs: bool,
    show_sidebar: bool,
    logger_state: tui_logger::TuiWidgetState,
}

impl Debug for RootComponent<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("RoomComponent")
            .field("message_field", &self.message_field)
            .field("active_text_area", &self.active_text_area)
            .field("checking_logs", &self.checking_logs)
            .field("show_sidebar", &self.show_sidebar)
            .field("logger_state", &"<LoggerState>")
            .finish()
    }
}

impl Default for RootComponent<'_> {
    fn default() -> Self {
        Self::new()
    }
}

fn text_area<'a>() -> TextArea<'a> {
    let mut input = TextArea::new(vec![]);
    input.set_tab_length(2);
    input.set_max_histories(0);
    input.set_block(Block::default().borders(Borders::ALL));
    input
}

fn layout() -> Layout {
    Layout::default().constraints([
        Constraint::Length(2),
        Constraint::Min(1),
        Constraint::Length(3),
    ])
}

/// Specifies the active text area
/// with the exception of the message input field
#[derive(Debug)]
enum ActiveTextArea<'a> {
    UsernameField(TextArea<'a>),
    #[allow(unused)]
    Popup(TextArea<'a>),
}

impl ActiveTextArea<'_> {
    fn input(&mut self, input: Input) -> bool {
        match self {
            ActiveTextArea::UsernameField(ta) | ActiveTextArea::Popup(ta) => ta.input(input),
        }
    }
}

impl Component for RootComponent<'_> {
    fn handle_event(&mut self, event: &Event, ctx: &mut AppContext) -> EventResult {
        if self.checking_logs {
            self.handle_log_input(event.clone())
        } else {
            self.handle_chat_input(event.clone(), ctx)
        }
    }

    fn render(&self, f: &mut Frame<'_>, area: Rect, ctx: &AppContext) {
        if self.checking_logs {
            draw_logs(f, area, &self.logger_state);
        } else {
            self.draw_chat(f, area, ctx);
        }
    }

    fn update(&self, ctx: &mut AppContext) {
        ctx.poll_room_events();
        ctx.send_sync_requests();
    }

    fn before_quit(&mut self, ctx: &mut AppContext) {
        ctx.rooms.values_mut().for_each(|room| room.quit());
    }
}

impl RootComponent<'_> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            active_text_area: None,
            message_field: text_area(),
            checking_logs: false,
            show_sidebar: false,
            logger_state: tui_logger::TuiWidgetState::default(),
        }
    }

    fn handle_log_input(&mut self, e: Event) -> EventResult {
        match e.into() {
            Input {
                key: Key::Char('l'),
                ctrl: true,
                ..
            } => {
                self.toggle_logs();
            }
            Input {
                key: Key::Char(' '),
                ..
            } => {
                self.logger_state.transition(TuiWidgetEvent::SpaceKey);
            }
            Input { key: Key::Esc, .. } => {
                self.logger_state.transition(TuiWidgetEvent::EscapeKey);
            }
            Input {
                key: Key::PageUp, ..
            } => {
                self.logger_state.transition(TuiWidgetEvent::PrevPageKey);
            }
            Input {
                key: Key::PageDown, ..
            } => {
                self.logger_state.transition(TuiWidgetEvent::NextPageKey);
            }
            Input { key: Key::Up, .. } => {
                self.logger_state.transition(TuiWidgetEvent::UpKey);
            }
            Input { key: Key::Down, .. } => {
                self.logger_state.transition(TuiWidgetEvent::DownKey);
            }
            Input { key: Key::Left, .. } => {
                self.logger_state.transition(TuiWidgetEvent::LeftKey);
            }
            Input {
                key: Key::Right, ..
            } => {
                self.logger_state.transition(TuiWidgetEvent::RightKey);
            }
            Input {
                key: Key::Char('+'),
                ..
            } => {
                self.logger_state.transition(TuiWidgetEvent::PlusKey);
            }
            Input {
                key: Key::Char('-'),
                ..
            } => {
                self.logger_state.transition(TuiWidgetEvent::MinusKey);
            }
            Input {
                key: Key::Char('h'),
                ..
            } => {
                self.logger_state.transition(TuiWidgetEvent::HideKey);
            }
            Input {
                key: Key::Char('f'),
                ..
            } => {
                self.logger_state.transition(TuiWidgetEvent::FocusKey);
            }
            Input {
                key: Key::Char('q'),
                ctrl: true,
                ..
            } => {
                return EventResult::Consumed(Some(AppAction::Quit));
            }
            _ => {
                // TODO: some other controls?
                return EventResult::Ignored;
            }
        }

        EventResult::Consumed(None)
    }

    fn handle_chat_input(&mut self, e: Event, ctx: &mut AppContext) -> EventResult {
        match e.into() {
            Input {
                key: Key::Char('n'),
                ctrl: true,
                ..
            } => {
                self.toggle_text_area(ctx);
            }
            Input {
                key: Key::Char('p'),
                ctrl: true,
                alt: false,
                ..
            } => {
                ctx.toggle_offset_mode();
            }
            Input {
                key: Key::Char('p'),
                ctrl: false,
                alt: true,
                ..
            } => {
                ctx.force_disable_offset();
            }
            Input {
                key: Key::Char('u'),
                ctrl: true,
                alt: false,
                ..
            } => {
                ctx.scroll_up();
            }
            Input {
                key: Key::Char('d'),
                ctrl: true,
                alt: false,
                ..
            } => {
                ctx.scroll_down();
            }
            Input {
                key: Key::Char('m'),
                ctrl: true,
                alt: false,
                ..
            }
            | Input {
                key: Key::Enter, ..
            } => {
                self.submit_text(ctx);
            }
            Input { key: Key::Esc, .. } => {
                self.exit_username_text_area();
            }
            Input {
                key: Key::Char('q'),
                ctrl: true,
                ..
            } => {
                return EventResult::Consumed(Some(AppAction::Quit));
            }
            Input {
                key: Key::Char('l'),
                ctrl: true,
                ..
            } => {
                self.toggle_logs();
            }
            Input {
                key: Key::Char('b'),
                ctrl: true,
                ..
            } => {
                self.toggle_sidebar();
            }
            input => {
                self.forward_input(input);
            }
        }

        EventResult::Consumed(None)
    }

    /// # Errors
    ///
    /// This function returns the Errors produced by `reqwest` client
    ///
    /// # Panics
    ///
    /// This function panics if the url can't be joined
    ///
    /// TODO: make this create a new modal which will dispatch the actual room join
    pub async fn join_room(
        &mut self,
        ctx: &mut AppContext,
        room_name: &str,
    ) -> anyhow::Result<tokio::task::JoinHandle<()>> {
        ctx.join_room(room_name).await
    }

    fn draw_chat(&self, f: &'_ mut Frame, area: Rect, ctx: &AppContext) {
        let x_constraints = if self.show_sidebar {
            #[allow(clippy::cast_possible_truncation)]
            let max_room_name = ctx
                .rooms
                .keys()
                .map(|r| r.chars().count())
                .max()
                .unwrap_or(5) as u16;
            let side_width = (max_room_name + 1).max(15);
            [Constraint::Length(side_width), Constraint::Fill(1)]
        } else {
            [Constraint::Length(0), Constraint::Fill(1)]
        };
        let x_areas = Layout::new(Direction::Horizontal, x_constraints)
            .spacing(Spacing::Overlap(u16::from(self.show_sidebar)))
            .split(area);
        let side_area = x_areas[0];
        let chat_area = x_areas[1];

        self.draw_sidebar(f, side_area, ctx);
        self.draw_room(f, chat_area, ctx, self.show_sidebar);
    }

    // Unused self is fine for now, there's probably going to be something in it that's needed
    #[allow(clippy::unused_self)]
    fn draw_sidebar(&self, f: &mut Frame, area: Rect, ctx: &AppContext) {
        let y_areas = Layout::new(
            Direction::Vertical,
            [Constraint::Length(2), Constraint::Fill(1)],
        )
        .split(area);
        let title_area = y_areas[0];
        let title = Paragraph::new("Rooms").block(
            Block::new()
                .borders(Borders::BOTTOM | Borders::RIGHT)
                .merge_borders(MergeStrategy::Fuzzy),
        );
        f.render_widget(title, title_area);

        let room_area = y_areas[1];
        let rooms = ctx.rooms.keys().map(String::as_str).collect::<Text>();
        let rooms = Paragraph::new(rooms).block(
            Block::new()
                .borders(Borders::RIGHT)
                .merge_borders(MergeStrategy::Fuzzy),
        );
        f.render_widget(rooms, room_area);
    }

    fn draw_room(&self, f: &mut Frame, area: Rect, ctx: &AppContext, left_borders: bool) {
        let borders = if left_borders {
            Borders::LEFT
        } else {
            Borders::NONE
        };
        let block = Block::new()
            .borders(borders)
            .merge_borders(MergeStrategy::Fuzzy);
        f.render_widget(&block, area);
        let area = block.inner(area);

        let chunks = layout().split(area);
        let mut name = String::from("Not in a room");
        if let Some(room) = ctx.current_room() {
            name = room
                .self_user()
                .map_or("Loading...".to_owned(), |u| u.get_name().to_owned());

            draw_room_events(
                f,
                chunks[1],
                room.events(),
                room.users(),
                room.scroll_offset(),
            );
        } else {
            f.render_widget(Clear, chunks[1]);
        }

        if let Some(ActiveTextArea::UsernameField(ta)) = &self.active_text_area {
            f.render_widget(ta, chunks[0]);
        } else {
            draw_top_bar(f, chunks[0], name);
        }
        f.render_widget(&self.message_field, chunks[2]);
    }

    fn submit_text(&mut self, ctx: &mut AppContext) {
        match &self.active_text_area {
            Some(ActiveTextArea::UsernameField(ta)) => {
                let username = ta.lines()[0].clone();
                self.active_text_area = None;
                ctx.current_room_mut_action(|r| {
                    r.change_name(&username);
                });
            }
            Some(ActiveTextArea::Popup(_)) => todo!(),
            None => {
                let message = self.message_field.lines()[0].clone();
                self.message_field.clear();
                ctx.current_room_mut_action(|r| {
                    r.send_text(&message);
                });
            }
        }
    }

    fn forward_input(&mut self, input: Input) {
        if let Some(active) = &mut self.active_text_area {
            active.input(input);
        } else {
            self.message_field.input(input);
        }
    }

    fn toggle_text_area(&mut self, ctx: &mut AppContext) {
        if self.active_text_area.is_none() {
            self.enter_username_text_area(ctx);
        } else {
            self.exit_username_text_area();
        }
    }

    fn exit_username_text_area(&mut self) {
        self.active_text_area = None;
    }

    fn enter_username_text_area(&mut self, ctx: &mut AppContext) {
        let Some(room) = ctx.current_room() else {
            return;
        };
        if self.active_text_area.is_none() {
            let mut text_area = text_area();
            text_area.set_block(top_block());
            if let Some(usr) = room.self_user() {
                text_area.insert_str(usr.get_name());
            }
            self.active_text_area = Some(ActiveTextArea::UsernameField(text_area));
        }
    }

    fn toggle_logs(&mut self) {
        self.checking_logs = !self.checking_logs;
    }

    fn toggle_sidebar(&mut self) {
        self.show_sidebar = !self.show_sidebar;
    }

    pub fn quit(&mut self, ctx: &mut AppContext) {
        for room in ctx.rooms.values_mut() {
            room.quit();
        }
    }
}
