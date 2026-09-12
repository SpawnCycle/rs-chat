use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct RoomArgs {
    pub name: Option<String>,
}
