use chat_lib::discovery::Discovery;
use ratatui::{
    Frame,
    crossterm::event::Event,
    layout::{Constraint, Layout},
    widgets::{Block, Borders},
};
use ratatui_textarea::{Input, Key, TextArea};
use reqwest::Client;
use std::{collections::HashMap, sync::mpsc::sync_channel};
use tokio::sync::mpsc::channel;
use tui_logger::{TuiWidgetEvent, TuiWidgetState};

use crate::{
    chat::{draw_room_events, draw_top_bar, top_block},
    config::AppConfig,
    consts::CHANNEL_BUFFER_SIZE,
    logs::draw_logs,
    room::Room,
    ws_handler::{WsAction, WsEvent, WsHandler},
};

pub struct App<'a> {
    username_field: Option<TextArea<'a>>,
    message_field: TextArea<'a>,
    should_quit: bool,
    layout: Layout,
    checking_logs: bool,
    rooms: HashMap<String, Room>,
    current_room_name: Option<String>,
    config: AppConfig,
    logger_state: TuiWidgetState,
}

fn text_area<'a>() -> TextArea<'a> {
    let mut input = TextArea::new(vec![]);
    input.set_tab_length(2);
    input.set_max_histories(0);
    input.set_block(Block::default().borders(Borders::ALL));
    input
}

impl App<'_> {
    #[must_use]
    pub fn new(config: AppConfig) -> Self {
        Self {
            rooms: HashMap::new(),
            current_room_name: None,
            should_quit: false,
            message_field: text_area(),
            layout: Layout::default().constraints([
                Constraint::Length(2),
                Constraint::Min(1),
                Constraint::Length(3),
            ]),
            username_field: None,
            checking_logs: false,
            logger_state: TuiWidgetState::default(),
            config,
        }
    }

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
            input => {
                self.forward_input(input);
            }
        }
    }

    // TODO: move this to a more appropriate place
    ///
    /// # Errors
    ///
    /// This function errors if the room creation errors
    pub async fn send_and_wait_for_message_from_room(
        &mut self,
        room_name: &str,
        action: WsAction,
        check: impl FnMut(&WsEvent) -> bool,
    ) -> anyhow::Result<Option<WsEvent>> {
        let (mut room, _) = self.new_room(room_name).await?;
        room.send_action(action);
        let res = room.wait_for_message::<false>(check).await;
        room.send_action(WsAction::Quit);
        Ok(res)
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
        let (e_tx, e_rx) = channel::<WsEvent>(CHANNEL_BUFFER_SIZE);
        let (a_tx, a_rx) = sync_channel::<WsAction>(CHANNEL_BUFFER_SIZE);

        let client = Client::new();

        let discovery = client
            .get(
                self.config
                    .web
                    .url
                    .join("about")
                    .expect("The url should be correct"),
            )
            .send()
            .await?
            .json::<Discovery>()
            .await?;

        log::debug!("{discovery:?}");

        let web_config = self.config.web.clone();
        let room_string = room_name.to_string();
        let ws = tokio::spawn(async move {
            // TODO: move this to a user action
            let handler = WsHandler::new(e_tx, a_rx, web_config, room_string)
                .await
                .inspect_err(|err| log::error!("Fatal error during websocket connection: {err}"));
            log::trace!("Websocket handler started");
            let Ok(mut handler) = handler else {
                return; // Ok to return because handler is not initialized
            };

            while !handler.step().await {}

            handler.close().await;

            log::trace!("Websocket handler ended");
        });

        Ok((Room::new(room_name, a_tx, e_rx), ws))
    }

    /// # Errors
    ///
    /// This function returns the Errors produced by `reqwest` client
    ///
    /// # Panics
    ///
    /// This function panics if the url can't be joined
    pub async fn room_discovery(&self, client: &Client) -> Result<Discovery, reqwest::Error> {
        client
            .get(
                self.config
                    .web
                    .url
                    .join("about")
                    .expect("The url should be correct"),
            )
            .send()
            .await?
            .json::<Discovery>()
            .await
    }

    pub fn draw(&self, f: &'_ mut Frame) {
        if self.checking_logs {
            draw_logs(f, f.area(), &self.logger_state);
        } else {
            self.draw_chat(f);
        }
    }

    fn draw_chat(&self, f: &'_ mut Frame) {
        let chunks = self.layout.split(f.area());
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
        }

        if let Some(area) = &self.username_field {
            f.render_widget(area, chunks[0]);
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
        if let Some(room) = self.current_room_mut() {
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
        let username = self.username_field.take();
        let message = self.message_field.lines()[0].clone();
        let Some(room) = self.current_room_mut() else {
            return;
        };
        if let Some(text_area) = username {
            room.change_name(&text_area.lines()[0]);
        } else {
            room.send_text(&message);
            self.message_field = text_area();
        }
    }

    fn forward_input(&mut self, input: Input) {
        if let Some(text_area) = &mut self.username_field {
            text_area.input(input);
        } else {
            self.message_field.input(input);
        }
    }

    fn toggle_text_area(&mut self) {
        if self.username_field.is_none() {
            self.enter_username_text_area();
        } else {
            self.exit_username_text_area();
        }
    }

    fn exit_username_text_area(&mut self) {
        self.username_field = None;
    }

    fn enter_username_text_area(&mut self) {
        let Some(room) = self.current_room() else {
            return;
        };
        if self.username_field.is_none() {
            let mut text_area = text_area();
            text_area.set_block(top_block());
            if let Some(usr) = room.self_user() {
                text_area.insert_str(usr.get_name());
            }
            self.username_field = Some(text_area);
        }
    }

    fn toggle_logs(&mut self) {
        self.checking_logs = !self.checking_logs;
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

impl std::fmt::Debug for App<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("App")
            .field("username_field", &self.username_field)
            .field("message_field", &self.message_field)
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
