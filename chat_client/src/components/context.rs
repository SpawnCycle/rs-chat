use std::collections::HashMap;

use crate::{config::AppConfig, helper::connect_room, room::Room, ws_handler::WsAction};

#[derive(Debug)]
pub struct AppContext {
    pub rooms: HashMap<String, Room>,
    pub current_room_name: Option<String>,
    pub config: AppConfig,
}

impl AppContext {
    #[must_use]
    pub fn new(config: AppConfig) -> Self {
        Self {
            rooms: HashMap::new(),
            current_room_name: None,
            config,
        }
    }

    /// # Errors
    ///
    /// This function errors if there was a problem either discovering the room
    /// or connecting to the associated websocket
    pub async fn join_room(
        &mut self,
        room_name: &str,
    ) -> anyhow::Result<tokio::task::JoinHandle<()>> {
        let (mut room, ws) = self.new_room(room_name).await?;
        // get ourselves
        room.send_action(WsAction::RequestSelf);
        // get all the users
        room.send_action(WsAction::RequestAll);
        self.rooms.insert(room_name.to_string(), room);

        if self.current_room_name.is_none() {
            self.current_room_name = Some(room_name.to_string());
        }

        // TODO: store the join handles in a different place
        Ok(ws)
    }

    async fn new_room(
        &self,
        room_name: &str,
    ) -> anyhow::Result<(Room, tokio::task::JoinHandle<()>)> {
        connect_room(self.config.web.clone(), &self.config.web.url, room_name).await
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

    #[must_use]
    pub fn current_room(&self) -> Option<&Room> {
        self.current_room_name
            .as_ref()
            .and_then(|r| self.rooms.get(r))
    }

    #[must_use]
    pub fn current_room_name(&self) -> Option<&str> {
        self.current_room_name.as_deref()
    }

    pub fn current_room_mut(&mut self) -> Option<&mut Room> {
        self.current_room_name
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
