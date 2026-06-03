use std::{collections::HashMap, num::NonZero, sync::mpsc::SyncSender};

use chat_lib::types::{Message, User};
use tokio::sync::mpsc::Receiver;
use uuid::Uuid;

use crate::{
    chat::Offset,
    event::{RoomEvent, UserLocator},
    ws_handler::{WsAction, WsEvent},
};

fn rel_to_abs(rel: usize, n: u32) -> u32 {
    #[allow(clippy::cast_possible_truncation)]
    let rel = (rel as u32).saturating_sub(1);
    rel - n.min(rel)
}

fn abs_to_rel(ev: usize, n: u32) -> Option<NonZero<u32>> {
    #[allow(clippy::cast_possible_truncation)]
    let abs = (ev as u32).saturating_sub(1);
    let rel = abs - n.min(abs);
    NonZero::new(rel)
}

#[derive(Debug)]
pub struct Room {
    events: Vec<RoomEvent>,
    self_id: Option<Uuid>,
    users: HashMap<Uuid, User>,
    scoll_offset: Option<Offset>,
    name: String,
    tx: SyncSender<WsAction>,
    rx: Receiver<WsEvent>,
    active: bool,
}

impl Room {
    #[must_use]
    pub fn new(name: &str, tx: SyncSender<WsAction>, rx: Receiver<WsEvent>) -> Self {
        Self {
            tx,
            rx,
            events: Vec::new(),
            users: HashMap::new(),
            self_id: None,
            scoll_offset: None,
            name: name.to_string(),
            active: true,
        }
    }

    pub fn active(&self) -> bool {
        self.active
    }

    pub fn users(&self) -> &HashMap<Uuid, User> {
        &self.users
    }

    pub fn scroll_offset(&self) -> Option<Offset> {
        self.scoll_offset
    }

    pub fn events(&self) -> &[RoomEvent] {
        &self.events
    }

    pub fn self_user(&self) -> Option<&User> {
        self.self_id.and_then(|id| self.get_user(id))
    }

    pub fn poll_pending_events(&mut self) {
        while let Ok(action) = self.rx.try_recv() {
            self.handle_event(action);
        }
    }

    pub fn send_sync_requests(&mut self) {
        if self.self_user().is_none() {
            self.send_action(WsAction::RequestSelf);
        }
        let message_events = self
            .events
            .iter()
            .filter_map(|e| match e {
                RoomEvent::Message(message) => Some(message),
                _ => None,
            })
            .cloned()
            .collect::<Vec<_>>();

        for msg in message_events {
            let id = *msg.get_author();
            if self.users.get_user(id).is_none() {
                self.send_action(WsAction::RequestUser(id));
            }
        }
    }

    pub fn quit(&mut self) {
        log::info!("Quit room {}", self.name);
        self.active = false;
        self.send_action(WsAction::Quit);
    }

    pub fn send_text(&mut self, text: &str) {
        let text = text.trim();
        if text.chars().count() > 0 {
            self.send_action(WsAction::Message(text.to_string()));
        }
    }

    pub fn change_name(&mut self, name: &str) {
        let name = name.trim();
        if name.chars().count() > 0 {
            self.send_action(WsAction::ChangeName(name.to_string()));
            self.send_action(WsAction::RequestSelf);
        }
    }

    pub fn scroll_up(&mut self) {
        match self.scoll_offset {
            None => {
                self.scoll_offset = Some(Offset::Relative(
                    NonZero::new(1).expect("Literal is non zero"),
                ));
            }
            Some(Offset::Absolute(n)) => {
                let offset = n.saturating_sub(1);
                self.scoll_offset = Some(Offset::Absolute(offset));
            }
            #[allow(clippy::cast_possible_truncation)]
            Some(Offset::Relative(n)) => {
                let offset = n.saturating_add(1);
                self.scoll_offset = NonZero::new(self.events.len() as u32)
                    .map(|ev| offset.min(ev))
                    .map(Offset::Relative);
            }
        }
    }

    pub fn scroll_down(&mut self) {
        match self.scoll_offset {
            None => {}
            #[allow(clippy::cast_possible_truncation)]
            Some(Offset::Absolute(n)) => {
                let offset = n.saturating_add(1).min(self.events.len() as u32);
                self.scoll_offset = Some(Offset::Absolute(offset));
            }
            Some(Offset::Relative(n)) => {
                let offset = n.get().saturating_sub(1);
                self.scoll_offset = NonZero::new(offset).map(Offset::Relative);
            }
        }
    }

    pub fn force_disable_offset(&mut self) {
        self.scoll_offset = None;
    }

    pub fn toggle_offset_mode(&mut self) {
        match self.scoll_offset {
            Some(offset) => match offset {
                Offset::Absolute(n) => {
                    #[allow(clippy::cast_possible_truncation)]
                    let rel = abs_to_rel(self.events.len(), n);
                    match rel {
                        Some(rel) => self.scoll_offset = Some(Offset::Relative(rel)),
                        None => self.scoll_offset = None,
                    }
                }
                Offset::Relative(n) => {
                    #[allow(clippy::cast_possible_truncation)]
                    let event_count = rel_to_abs(self.events.len(), n.get());
                    let abs = event_count - n.get().min(event_count);
                    if abs == 0 {
                        self.scoll_offset = None;
                    } else {
                        self.scoll_offset = Some(Offset::Absolute(abs));
                    }
                }
            },
            #[allow(clippy::cast_possible_truncation)]
            None => {
                self.scoll_offset = Some(Offset::Absolute(
                    (self.events.len() as u32).saturating_sub(1),
                ));
            }
        }
    }

    fn handle_event(&mut self, event: WsEvent) {
        match event {
            WsEvent::UserAdd(user) => {
                self.add_user(user);
            }
            WsEvent::UserRemove(uuid) => {
                self.remove_user(uuid);
            }
            WsEvent::UserChange(user) => {
                self.change_user_name(&user);
            }
            WsEvent::Message(message) => {
                self.add_message(message);
            }
            WsEvent::Quit => {
                self.quit();
            }
            WsEvent::SelfInfo(user) => {
                self.set_self(user);
            }
            WsEvent::UserInfo(user) => {
                self.set_user(user);
            }
            WsEvent::Banned(duration, reason) => {
                self.add_event(RoomEvent::Banned { duration, reason });
            }
            WsEvent::UserAllInfo(users) => {
                for user in users {
                    self.set_user(user);
                }
            }
        }
    }

    // TODO: Handle the errors gracefully
    pub(crate) fn send_action(&mut self, action: WsAction) {
        let _ = self.tx.send(action);
    }

    fn add_user(&mut self, user: User) -> bool {
        if self.users.contains_key(user.get_id()) {
            true
        } else {
            self.add_event(RoomEvent::UserJoined(*user.get_id()));
            self.set_user(user);
            false
        }
    }

    fn set_user(&mut self, user: User) {
        self.users.insert(*user.get_id(), user);
    }

    fn get_user(&self, id: Uuid) -> Option<&User> {
        self.users.get_user(id)
    }

    fn remove_user(&mut self, id: Uuid) {
        // do not remove the user to keep all the references alive
        // self.users.remove(id);
        self.add_event(RoomEvent::UserLeft(id));
    }

    fn set_self(&mut self, usr: User) {
        self.self_id = Some(*usr.get_id());
        self.set_user(usr);
    }

    fn change_user_name(&mut self, usr: &User) {
        let id = usr.get_id();
        let name = usr.get_name();
        self.add_event(RoomEvent::UserNameChange {
            from: self
                .get_user(*id)
                .map(|u| u.get_name().to_owned())
                .unwrap_or(id.to_string()),
            to: name.to_owned(),
        });
        self.users
            .entry(*usr.get_id())
            .and_modify(|u| u.set_name(usr.get_name().to_string()));
    }

    fn add_message(&mut self, msg: Message) {
        self.add_event(msg);
    }

    fn add_event(&mut self, ev: impl Into<RoomEvent>) {
        self.events.push(ev.into());
    }
}
