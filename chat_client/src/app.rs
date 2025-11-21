use std::sync::mpsc::SyncSender;

use chat_lib::types::{Message, User};

use ratatui::{
    Frame,
    crossterm::event::Event,
    layout::{Constraint, Layout},
    widgets::{Block, Borders, Paragraph, Row, Table},
};
use tui_textarea::{Input, Key, TextArea};
use uuid::Uuid;

use crate::{
    room_event::RoomEvent,
    ws_handler::{WsAction, WsEvent},
};

pub struct App<'a> {
    username_field: Option<TextArea<'a>>,
    message_field: TextArea<'a>,
    room_events: Vec<RoomEvent>,
    should_quit: bool,
    self_id: Option<Uuid>,
    layout: Layout,
    users: Vec<User>,
    tx: SyncSender<WsAction>,
}

fn text_area<'a>() -> TextArea<'a> {
    let mut input = TextArea::new(vec![]);
    input.set_tab_length(2);
    input.set_max_histories(0);
    input.set_block(Block::default().borders(Borders::ALL));
    input
}

fn top_block<'a>() -> Block<'a> {
    Block::new().borders(Borders::BOTTOM)
}

impl<'a> App<'a> {
    pub fn new(tx: SyncSender<WsAction>) -> Self {
        let users = Vec::new();
        Self {
            tx,
            users,
            should_quit: false,
            room_events: vec![],
            message_field: text_area(),
            layout: Layout::default().constraints([
                Constraint::Length(2),
                Constraint::Min(1),
                Constraint::Length(3),
            ]),
            self_id: None,
            username_field: None,
        }
    }

    pub fn handle_input(&mut self, e: Event) {
        match e.into() {
            Input {
                key: Key::Char('n'),
                ctrl: true,
                alt: false,
                ..
            } => {
                if self.username_field.is_none() {
                    let mut text_area = text_area();
                    text_area.set_block(top_block());
                    if let Some(usr) = self.get_self() {
                        text_area.insert_str(usr.get_name());
                    }
                    self.username_field = Some(text_area);
                } else {
                    self.username_field = None;
                }
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
                if let Some(text_area) = self.username_field.take() {
                    let text = text_area.lines()[0].clone();
                    if text.trim().chars().count() > 0 {
                        self.send_action(WsAction::ChangeName(text.to_owned()));
                        self.send_action(WsAction::RequestSelf);
                    }
                } else {
                    let text = self.message_field.lines()[0].clone();
                    if text.trim().chars().count() > 0 {
                        self.send_action(WsAction::Message(text.to_owned()));
                        self.message_field = text_area();
                    }
                }
            }
            Input {
                key: Key::Char('q'),
                ctrl: true,
                ..
            } => {
                self.quit();
            }
            input => {
                if let Some(text_area) = &mut self.username_field {
                    text_area.input(input);
                } else {
                    self.message_field.input(input);
                }
            }
        };
    }

    pub fn draw(&self, f: &'_ mut Frame) {
        let chunks = self.layout.split(f.area());
        let top_bar = Paragraph::new(
            self.get_self()
                .map(|u| u.get_name().to_owned())
                .unwrap_or("Loading...".to_owned()),
        )
        .block(Block::new().borders(Borders::BOTTOM));
        let rows = self
            .room_events()
            .map(|m| m.as_row(self.all_users()))
            .collect::<Vec<Row>>();

        let table = Table::new(rows, &[Constraint::Max(20), Constraint::Fill(1)]);

        if let Some(text_area) = &self.username_field {
            f.render_widget(text_area, chunks[0]);
        } else {
            f.render_widget(&top_bar, chunks[0]);
        }
        f.render_widget(&table, chunks[1]);
        f.render_widget(&self.message_field, chunks[2]);
    }

    // TODO: Handle the errors gracefully
    pub fn send_action(&mut self, action: WsAction) {
        let _ = self.tx.send(action);
    }

    pub fn handle_event(&mut self, action: &WsEvent) {
        match action {
            WsEvent::UserAdd(user) => {
                log::info!("Action: Add User");
                self.add_user(user);
            }
            WsEvent::UserRemove(uuid) => {
                log::info!("Action: Remove User");
                self.remove_user(uuid);
            }
            WsEvent::UserChange(user) => {
                log::info!("Action: Change User");
                self.change_user_name(user);
            }
            WsEvent::Message(message) => {
                log::info!("Action: Add Message");
                self.add_message(message);
            }
            WsEvent::Quit => {
                log::info!("Action: Quit");
                self.quit()
            }
            WsEvent::SelfInfo(user) => {
                log::info!("Action: SelfInfo");
                self.set_self(user);
            }
            WsEvent::UserInfo(user) => {
                log::info!("Action: UserInfo");
                self.set_user(user);
            }
        }
    }

    pub fn send_sync_requests(&mut self) {
        if self.get_self().is_none() {
            self.send_action(WsAction::RequestSelf);
        }

        self.room_events
            .iter()
            .filter_map(|e| match e {
                RoomEvent::Message(message) => Some(message),
                _ => None,
            })
            .cloned()
            // as far as I'm aware the only way of having self both in
            // the filter and the foreach
            .collect::<Vec<Message>>()
            .iter()
            .for_each(|msg| {
                let id = *msg.get_author();
                if !self.users.iter().any(|usr| usr.get_id() == &id)
                    && Some(&id) != self.self_id.as_ref()
                {
                    self.send_action(WsAction::RequestUser(id));
                }
            });
    }

    pub fn get_self(&self) -> Option<&User> {
        self.self_id.and_then(|id| self.get_user(&id))
    }

    pub fn room_events(&self) -> impl Iterator<Item = &RoomEvent> {
        self.room_events.iter()
    }

    pub fn add_user(&mut self, user: &User) -> bool {
        if self.users.iter().any(|usr| usr == user) {
            true
        } else {
            self.room_events.push(RoomEvent::UserJoined(*user.get_id()));
            self.set_user(user);
            false
        }
    }

    pub fn set_user(&mut self, usr: &User) {
        if self.users.contains(usr) {
            self.users.iter_mut().for_each(|u| {
                if u.get_id() == usr.get_id() {
                    u.clone_from(usr);
                }
            });
        } else {
            self.users.push(usr.clone());
        }
    }

    pub fn all_users(&self) -> &Vec<User> {
        &self.users
    }

    pub fn get_user(&self, id: &Uuid) -> Option<&User> {
        self.users.iter().find(|u| u.get_id() == id)
    }

    pub fn remove_user(&mut self, id: &Uuid) {
        // do not remove the user to keep all the references alive
        // self.users.retain(|usr| usr.get_id() != id);
        self.room_events.push(RoomEvent::UserLeft(*id));
    }

    pub fn set_self(&mut self, usr: &User) {
        self.self_id = Some(*usr.get_id());
        self.set_user(usr);
    }

    pub fn change_user_name(&mut self, usr: &User) {
        let id = usr.get_id();
        let name = usr.get_name();
        self.room_events.push(RoomEvent::UserNameChange {
            from: self
                .get_user(id)
                .map(|u| u.get_name().to_owned())
                .unwrap_or(id.to_string()),
            to: name.to_owned(),
        });
        self.users.iter_mut().for_each(|u| {
            if u.get_id() == usr.get_id() {
                u.clone_from(usr);
            }
        });
    }

    pub fn add_message(&mut self, msg: &Message) {
        self.room_events.push(msg.clone().into());
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
        let _ = self.tx.send(WsAction::Quit);
    }
}
