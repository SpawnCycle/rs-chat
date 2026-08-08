use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Display)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum Version {
    V1,
    V2,
    V3,
}
