use ratatui::{
    Frame,
    crossterm::event::Event,
    layout::{Constraint, Direction, Layout, Rect, Spacing},
    symbols::merge::MergeStrategy,
    text::Text,
    widgets::{Block, Borders, Clear, Paragraph},
};
use ratatui_textarea::{Input, Key, TextArea};
use reqwest::Client;
use std::{collections::HashMap, sync::mpsc::sync_channel};
use tokio::sync::mpsc::channel;
use tui_logger::TuiWidgetEvent;
use url::Url;

use crate::{
    chat::{draw_room_events, draw_top_bar, top_block},
    config::{AppConfig, WebConfig},
    consts::CHANNEL_BUFFER_SIZE,
    logs::draw_logs,
    requests::room_discovery,
    room::Room,
    ws_handler::{WsAction, WsEvent, WsHandler},
};

// TODO: rethink the app structure to allow more flexibility
pub struct App<'a> {
    message_field: TextArea<'a>,
    active_text_area: Option<ActiveTextArea<'a>>,
    should_quit: bool,
    layout: Layout,
    checking_logs: bool,
    show_sidebar: bool,
    rooms: HashMap<String, Room>,
    current_room_name: Option<String>,
    config: AppConfig,
    logger_state: tui_logger::TuiWidgetState,
}

impl std::fmt::Debug for App<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("App")
            .field("message_field", &self.message_field)
            .field("active_text_area", &self.active_text_area)
            .field("should_quit", &self.should_quit)
            .field("layout", &self.layout)
            .field("checking_logs", &self.checking_logs)
            .field("rooms", &self.rooms)
            .field("current_room_name", &self.current_room_name)
            .field("config", &self.config)
            .field("logger_state", &"<LoggerState>")
            .finish()
    }
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

fn text_area<'a>() -> TextArea<'a> {
    let mut input = TextArea::new(vec![]);
    input.set_tab_length(2);
    input.set_max_histories(0);
    input.set_block(Block::default().borders(Borders::ALL));
    input
}

/// # Errors
///
/// This function errors if there was a problem during any of the web calls,
/// be it an internet error or a parse one
pub async fn connect_room(
    config: WebConfig,
    base_url: &Url,
    room_name: &str,
) -> anyhow::Result<(Room, tokio::task::JoinHandle<()>)> {
    let (e_tx, e_rx) = channel::<WsEvent>(CHANNEL_BUFFER_SIZE);
    let (a_tx, a_rx) = sync_channel::<WsAction>(CHANNEL_BUFFER_SIZE);

    let client = Client::new();

    let discovery = room_discovery(&client, base_url).await?;

    log::debug!("{discovery:?}");

    let web_config = config.clone();
    let room_string = room_name.to_string();
    let ws = tokio::spawn(async move {
        let handler = WsHandler::new(e_tx, a_rx, web_config, room_string.clone())
            .await
            .inspect_err(|err| log::error!("Fatal error during websocket connection: {err}"));
        log::debug!("Websocket handler for {room_string} started");
        let Ok(mut handler) = handler else {
            return; // Ok to return because handler is not initialized
        };

        while !handler.step().await {}

        handler.close().await;

        log::debug!("Websocket handler for {room_string} ended");
    });

    Ok((Room::new(room_name, a_tx, e_rx), ws))
}

impl App<'_> {
    #[must_use]
    pub fn new(config: AppConfig) -> Self {
        Self {
            active_text_area: None,
            rooms: HashMap::new(),
            current_room_name: None,
            should_quit: false,
            message_field: text_area(),
            layout: Layout::default().constraints([
                Constraint::Length(2),
                Constraint::Min(1),
                Constraint::Length(3),
            ]),
            checking_logs: false,
            show_sidebar: false,
            logger_state: tui_logger::TuiWidgetState::default(),
            config,
        }
    }

    // TODO: rethink input handling
    pub fn handle_input(&mut self, e: Event) {
        if self.checking_logs {
            self.handle_log_input(e);
        } else {
            self.handle_chat_input(e);
        }
    }

    pub fn handle_log_input(&mut self, e: Event) {
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
                self.quit();
            }
            _ => {
                // TODO: some other controls?
            }
        }
    }

    pub fn handle_chat_input(&mut self, e: Event) {
        match e.into() {
            Input {
                key: Key::Char('n'),
                ctrl: true,
                ..
            } => {
                self.toggle_text_area();
            }
            Input {
                key: Key::Char('p'),
                ctrl: true,
                alt: false,
                ..
            } => {
                self.toggle_offset_mode();
            }
            Input {
                key: Key::Char('p'),
                ctrl: false,
                alt: true,
                ..
            } => {
                self.force_disable_offset();
            }
            Input {
                key: Key::Char('u'),
                ctrl: true,
                alt: false,
                ..
            } => {
                self.scroll_up();
            }
            Input {
                key: Key::Char('d'),
                ctrl: true,
                alt: false,
                ..
            } => {
                self.scroll_down();
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
                self.submit_text();
            }
            Input { key: Key::Esc, .. } => {
                self.exit_username_text_area();
            }
            Input {
                key: Key::Char('q'),
                ctrl: true,
                ..
            } => {
                self.quit();
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
    }

    /// # Errors
    ///
    /// This function returns the Errors produced by `reqwest` client
    ///
    /// # Panics
    ///
    /// This function panics if the url can't be joined
    pub async fn join_room(
        &mut self,
        room_name: &str,
    ) -> anyhow::Result<tokio::task::JoinHandle<()>> {
        let (room, ws) = self.new_room(room_name).await?;
        self.rooms.insert(room_name.to_string(), room);

        if self.current_room_name.is_none() {
            self.current_room_name = Some(room_name.to_string());
        }

        // TODO: store the join handles in a different place
        Ok(ws)
    }

    // TODO: move the join handle to a different place
    async fn new_room(
        &self,
        room_name: &str,
    ) -> anyhow::Result<(Room, tokio::task::JoinHandle<()>)> {
        connect_room(self.config.web.clone(), &self.config.web.url, room_name).await
    }

    pub fn draw(&self, f: &'_ mut Frame) {
        if self.checking_logs {
            draw_logs(f, f.area(), &self.logger_state);
        } else {
            self.draw_chat(f);
        }
    }

    fn draw_chat(&self, f: &'_ mut Frame) {
        let x_constraints = if self.show_sidebar {
            #[allow(clippy::cast_possible_truncation)]
            let max_room_name = self
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
            .split(f.area());
        let side_area = x_areas[0];
        let chat_area = x_areas[1];

        self.draw_sidebar(f, side_area);
        self.draw_room(f, chat_area, self.show_sidebar);
    }

    fn draw_sidebar(&self, f: &mut Frame, area: Rect) {
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
        let rooms = self.rooms.keys().map(String::as_str).collect::<Text>();
        let rooms = Paragraph::new(rooms).block(
            Block::new()
                .borders(Borders::RIGHT)
                .merge_borders(MergeStrategy::Fuzzy),
        );
        f.render_widget(rooms, room_area);
    }

    fn draw_room(&self, f: &mut Frame, area: Rect, left_borders: bool) {
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

        let chunks = self.layout.split(area);
        let mut name = String::from("Not in a room");
        if let Some(room) = self.current_room() {
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

    pub fn poll_room_events(&mut self) {
        for room in self.rooms.values_mut() {
            room.poll_pending_events();
        }
        self.rooms.retain(|_, room| room.active());

        if let Some(current_room) = &self.current_room_name
            && !self.rooms.contains_key(current_room)
        {
            self.current_room_name = None;
        }
    }

    pub fn send_sync_requests(&mut self) {
        for room in self.rooms.values_mut() {
            room.send_sync_requests();
        }
    }

    fn scroll_up(&mut self) {
        if let Some(room) = self.current_room_mut() {
            room.scroll_up();
        }
    }

    fn scroll_down(&mut self) {
        if let Some(room) = self.current_room_mut() {
            room.scroll_down();
        }
    }

    fn current_room(&self) -> Option<&Room> {
        self.current_room_name
            .clone()
            .and_then(|r| self.rooms.get(&r))
    }

    fn current_room_mut(&mut self) -> Option<&mut Room> {
        self.current_room_name
            .clone()
            .and_then(|r| self.rooms.get_mut(&r))
    }

    fn current_room_mut_action(&mut self, f: impl FnOnce(&mut Room)) {
        if let Some(room) = self.current_room_mut() {
            f(room);
        }
    }

    fn toggle_offset_mode(&mut self) {
        if let Some(room) = self.current_room_mut() {
            room.toggle_offset_mode();
        }
    }

    fn force_disable_offset(&mut self) {
        if let Some(room) = self.current_room_mut() {
            room.force_disable_offset();
        }
    }

    fn submit_text(&mut self) {
        match &self.active_text_area {
            Some(ActiveTextArea::UsernameField(ta)) => {
                let username = ta.lines()[0].clone();
                self.active_text_area = None;
                self.current_room_mut_action(|r| {
                    r.change_name(&username);
                });
            }
            Some(ActiveTextArea::Popup(_)) => todo!(),
            None => {
                let message = self.message_field.lines()[0].clone();
                self.message_field.clear();
                self.current_room_mut_action(|r| {
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

    fn toggle_text_area(&mut self) {
        if self.active_text_area.is_none() {
            self.enter_username_text_area();
        } else {
            self.exit_username_text_area();
        }
    }

    fn exit_username_text_area(&mut self) {
        self.active_text_area = None;
    }

    fn enter_username_text_area(&mut self) {
        let Some(room) = self.current_room() else {
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

    pub fn quit(&mut self) {
        self.should_quit = true;
        for room in self.rooms.values_mut() {
            room.quit();
        }
    }

    // getters/setters

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }
}
