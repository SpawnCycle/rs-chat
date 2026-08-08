use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Version {
    // The only one implemented currently
    V1,
    V2,
    V3,
}
