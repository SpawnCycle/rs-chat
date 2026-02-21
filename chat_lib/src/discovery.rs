use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Discovery {
    pub version: String,
    pub available_rooms: Vec<String>,
}
