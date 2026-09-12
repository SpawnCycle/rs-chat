use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Discovery {
    pub server_version: semver::Version,
    pub available_rooms: Vec<String>,
    pub supported_api_versions: Vec<crate::Version>,
}
