use std::{collections::HashMap, time::Instant};

use chat_lib::Discovery;
use tokio::sync::mpsc::{Receiver, Sender, channel};
use url::Url;

use crate::{
    config::AppConfig,
    consts::CHANNEL_BUFFER_SIZE,
    helper::{FetchState, RoomLocation, connect_room_ws},
    room::Room,
    task::{AppTaskPayload, AppTaskResult, start_discovery},
};

#[derive(Debug)]
pub struct AppContext {
    pub rooms: HashMap<RoomLocation, Room>,
    pub current_room_location: Option<RoomLocation>,
    pub config: AppConfig,
    pub task_rx: Receiver<AppTaskResult>,
    /// We need to store this in order to create new senders
    pub task_tx: Sender<AppTaskResult>,
    pub discoveries: HashMap<Url, (FetchState<Discovery, String>, Instant)>,
    pub join_queue: Vec<RoomLocation>,
}

impl AppContext {
    #[must_use]
    pub fn new(config: AppConfig) -> Self {
        let (tx, rx) = channel(CHANNEL_BUFFER_SIZE);

        Self {
            rooms: HashMap::new(),
            current_room_location: None,
            config,
            task_rx: rx,
            task_tx: tx,
            discoveries: HashMap::new(),
            join_queue: Vec::new(),
        }
    }

    pub fn update(&mut self) {
        self.poll_room_events();
        self.send_sync_requests();
        self.poll_tasks();
        self.process_join_queue();
    }

    pub fn quit_all_rooms(&mut self) {
        self.rooms.values_mut().for_each(Room::quit);
    }

    pub fn join_room(&mut self, base: Url, room_name: impl ToString) {
        self.join_queue.push(RoomLocation {
            url: base,
            room_name: room_name.to_string(),
        });
    }

    fn connect_room(&mut self, loc: RoomLocation) {
        if self.rooms.contains_key(&loc) {
            // We're already in a room, this request is outdated
            return;
        }

        let (room, _) = self.new_room(loc.url.clone(), &loc.room_name);
        if let None = self.current_room_location {
            self.current_room_location = Some(loc.clone());
        }
        self.rooms.insert(loc, room);
    }

    fn new_room(&self, base: Url, room_name: &str) -> (Room, tokio::task::JoinHandle<()>) {
        connect_room_ws(self.config.web.clone(), &base, room_name)
    }

    pub fn discover(&mut self, url: Url) {
        if let Some(d) = self.discoveries.get(&url)
            && matches!(d.0, FetchState::Pending)
        {
            // It's already pending
            return;
        }

        start_discovery(self.task_tx.clone(), url.clone());
        self.discoveries
            .insert(url, (FetchState::Pending, Instant::now()));
    }

    pub fn poll_tasks(&mut self) {
        while let Ok(task) = self.task_rx.try_recv() {
            self.process_task(task.base, task.payload);
        }
    }

    pub fn process_join_queue(&mut self) {
        let mut new_queue = Vec::with_capacity(self.join_queue.len());

        // very sad, but I see no way around cloning it
        for loc in self.join_queue.clone() {
            if let Some(fetch) = self.discoveries.get(&loc.url) {
                let (state, _when) = fetch;
                match state {
                    FetchState::Pending => new_queue.push(loc),
                    FetchState::Value(_) => {
                        self.connect_room(loc);
                    }
                    FetchState::Error(err) => {
                        log::error!("Tried to join bad server {err}");
                    }
                }
            } else {
                self.discover(loc.url.clone());
                new_queue.push(loc);
            }
        }

        self.join_queue = new_queue;
    }

    fn process_task(&mut self, base: Url, task: AppTaskPayload) {
        let now = Instant::now();
        match task {
            AppTaskPayload::Discovery(discovery) => match discovery {
                Ok(val) => {
                    self.discoveries.insert(base, (FetchState::Value(val), now));
                }
                Err(err) => {
                    self.discoveries
                        .insert(base, (FetchState::Error(err.to_string()), now));
                }
            },
        }
    }

    pub fn poll_room_events(&mut self) {
        for room in self.rooms.values_mut() {
            room.poll_pending_events();
        }
        self.rooms.retain(|_, room| room.active());

        if let Some(current_room) = &self.current_room_location
            && !self.rooms.contains_key(current_room)
        {
            self.current_room_location = None;
        }
    }

    pub fn send_sync_requests(&mut self) {
        for room in self.rooms.values_mut() {
            room.send_sync_requests();
        }
    }

    pub fn get_discovery(&self, url: &Url) -> (FetchState<Discovery, String>, Instant) {
        self.discoveries
            .get(url)
            .cloned()
            .unwrap_or((FetchState::Pending, Instant::now()))
    }

    #[must_use]
    pub fn current_room(&self) -> Option<&Room> {
        self.current_room_location
            .as_ref()
            .and_then(|r| self.rooms.get(r))
    }

    #[must_use]
    pub fn current_room_name(&self) -> Option<&str> {
        self.current_room_location
            .as_ref()
            .map(|r| r.room_name.as_str())
    }

    pub fn current_room_mut(&mut self) -> Option<&mut Room> {
        self.current_room_location
            .clone()
            .and_then(|r| self.rooms.get_mut(&r))
    }

    pub fn current_room_mut_action(&mut self, f: impl FnOnce(&mut Room)) {
        if let Some(room) = self.current_room_mut() {
            f(room);
        }
    }

    pub fn toggle_offset_mode(&mut self) {
        if let Some(room) = self.current_room_mut() {
            room.toggle_offset_mode();
        }
    }

    pub fn force_disable_offset(&mut self) {
        if let Some(room) = self.current_room_mut() {
            room.force_disable_offset();
        }
    }

    pub fn scroll_up(&mut self) {
        if let Some(room) = self.current_room_mut() {
            room.scroll_up();
        }
    }

    pub fn scroll_down(&mut self) {
        if let Some(room) = self.current_room_mut() {
            room.scroll_down();
        }
    }
}
