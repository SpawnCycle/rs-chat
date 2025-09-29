use std::{collections::HashMap, sync::mpsc::SyncSender};

use eszi_lib::messages::types::{Message, User};

use ratatui::{
    Frame,
    crossterm::event::Event,
    layout::{Constraint, Layout},
    widgets::{Block, Borders, Cell, Row, Table},
};
use tui_textarea::{Input, Key, TextArea};
use uuid::Uuid;

use crate::{TEST_UUID, client_utils::ws_handler::WsAction};

pub struct App<'a> {
    pub(crate) messages: Vec<Message>,
    users: HashMap<Uuid, String>,
    tx: SyncSender<WsAction>,
    input: TextArea<'a>,
    should_quit: bool,
    layout: Layout,
    my_uuid: Uuid,
}

fn text_area_template() -> TextArea<'static> {
    let mut input = TextArea::new(vec![]);
    input.set_tab_length(2);
    input.set_max_histories(0);
    input.set_block(Block::default().borders(Borders::ALL));
    input
}

impl<'a> App<'a> {
    pub fn new(tx: SyncSender<WsAction>) -> Self {
        let my_uuid = Uuid::new_v4();
        let mut users = HashMap::new();
        users.insert(TEST_UUID, "Test 123".to_string());
        Self {
            should_quit: false,
            messages: vec![],
            users: users,
            input: text_area_template(),
            layout: Layout::default().constraints([Constraint::Min(1), Constraint::Length(3)]),
            my_uuid,
            tx,
        }
    }

    pub async fn handle_event(&mut self, e: Event) {
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
                self.input = text_area_template();
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
        let rows = self
            .messages
            .iter()
            .map(|m| {
                if let Some(name) = self.users.get(m.get_author()) {
                    Row::new(vec![
                        Cell::new(name.to_string()),
                        Cell::new(m.get_content().to_string()),
                    ])
                } else {
                    return Row::new(vec![self.users.get(&TEST_UUID).unwrap(), m.get_content()]);
                }
            })
            .collect::<Vec<Row>>();

        let chunks = self.layout.split(f.area());

        let table = Table::new(rows, &[Constraint::Max(20), Constraint::Fill(1)]);
        f.render_widget(&table, chunks[0]);
        f.render_widget(&self.input, chunks[1]);
    }

    pub fn add_user(&mut self, usr: User) {
        self.users.insert(*usr.get_id(), usr.get_name().to_string());
    }

    pub fn remove_user(&mut self, id: Uuid) {
        self.users.remove(&id);
    }

    pub fn change_user_name(&mut self, usr: User) {
        let new_name = usr.get_name().to_string();
        self.users
            .entry(*usr.get_id())
            .and_modify(|v| *v = new_name.to_owned())
            .or_insert(new_name.to_owned());
    }

    pub fn add_message(&mut self, msg: Message) {
        self.messages.push(msg);
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
        let _ = self.tx.send(WsAction::Quit);
    }
}
