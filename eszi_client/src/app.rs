use std::{collections::HashMap, sync::mpsc::SyncSender};

use eszi_lib::types::{Message, User};

use ratatui::{
    Frame,
    crossterm::event::Event,
    layout::{Constraint, Layout},
    style::Stylize,
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};
use tui_textarea::{Input, Key, TextArea};
use uuid::Uuid;

use crate::ws_handler::{WsAction, WsEvent};

pub struct App<'a> {
    messages: Vec<Message>,
    users: HashMap<Uuid, String>,
    tx: SyncSender<WsAction>,
    input: TextArea<'a>,
    should_quit: bool,
    layout: Layout,
    self_user: Option<User>,
}

fn text_area<'a>() -> TextArea<'a> {
    let mut input = TextArea::new(vec![]);
    input.set_tab_length(2);
    input.set_max_histories(0);
    input.set_block(Block::default().borders(Borders::ALL));
    input
}

impl<'a> App<'a> {
    pub fn new(tx: SyncSender<WsAction>) -> Self {
        let users = HashMap::new();
        Self {
            should_quit: false,
            messages: vec![],
            users,
            input: text_area(),
            layout: Layout::default().constraints([
                Constraint::Length(2),
                Constraint::Min(1),
                Constraint::Length(3),
            ]),
            self_user: None,
            tx,
        }
    }

    pub fn handle_event(&mut self, e: Event) {
        match e.into() {
            Input {
                key: Key::Char('m'),
                ctrl: true,
                alt: false,
                ..
            }
            | Input {
                key: Key::Enter, ..
            } => {
                let text = self.input.lines()[0].clone();
                let _ = self.tx.send(WsAction::Message(text.to_owned()));
                self.input = text_area();
            }
            Input { key: Key::Esc, .. }
            | Input {
                key: Key::Char('c'),
                ctrl: true,
                ..
            } => {
                self.quit();
            }
            input => {
                self.input.input(input);
            }
        };
    }

    pub fn draw(&self, f: &'_ mut Frame) {
        let chunks = self.layout.split(f.area());
        let top_bar = Paragraph::new(
            self.get_self()
                .map(|usr| usr.get_name().to_owned())
                .unwrap_or("Loading...".to_owned()),
        )
        .block(Block::new().borders(Borders::BOTTOM));
        let rows = self
            .messages()
            .map(|m| {
                if let Some(name) = self.users.get(m.get_author()) {
                    Row::new(vec![
                        Cell::new(name.to_string()),
                        Cell::new(m.get_content()),
                    ])
                } else {
                    Row::new(vec![
                        Cell::new("Loading...").dim(),
                        Cell::new(m.get_content()),
                    ])
                }
            })
            .collect::<Vec<Row>>();

        let table = Table::new(rows, &[Constraint::Max(20), Constraint::Fill(1)]);

        f.render_widget(&top_bar, chunks[0]);
        f.render_widget(&table, chunks[1]);
        f.render_widget(&self.input, chunks[2]);
    }

    pub fn handle_action(&mut self, action: &WsEvent) {
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
        if self.self_user.is_none() {
            let _ = self.tx.send(WsAction::RequestSelf);
        }
        for msg in self.messages.iter() {
            let id = *msg.get_author();
            if let None = self.users.get(&id)
                && Some(&id) != self.self_user.as_ref().map(|usr| usr.get_id())
            {
                let _ = self.tx.send(WsAction::RequestUser(id));
            }
        }
    }

    pub fn get_self(&self) -> Option<User> {
        if let Some(id) = self.self_user.as_ref().map(|usr| usr.get_id()) {
            self.users
                .get(id)
                .map(|name| User::new(*id, name.to_owned()))
        } else {
            None
        }
    }

    pub fn messages(&self) -> impl Iterator<Item = &Message> {
        self.messages.iter()
    }

    pub fn add_user(&mut self, usr: &User) -> bool {
        if self.users.contains_key(usr.get_id()) {
            true
        } else {
            self.set_user(usr);
            false
        }
    }

    pub fn set_user(&mut self, usr: &User) {
        self.users
            .entry(*usr.get_id())
            .and_modify(|v| *v = usr.get_name().to_owned())
            .or_insert(usr.get_name().to_owned());
    }

    pub fn remove_user(&mut self, id: &Uuid) {
        self.users.remove(id);
    }

    pub fn set_self(&mut self, usr: &User) {
        self.self_user = Some(usr.clone());
        self.set_user(usr);
    }

    pub fn change_user_name(&mut self, usr: &User) {
        let new_name = usr.get_name().to_string();
        self.users
            .entry(*usr.get_id())
            .and_modify(|v| *v = new_name.to_owned())
            .or_insert(new_name.to_owned());
    }

    pub fn add_message(&mut self, msg: &Message) {
        self.messages.push(msg.clone());
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
        let _ = self.tx.send(WsAction::Quit);
    }
}
