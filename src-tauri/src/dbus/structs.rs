use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct OpenVPN3Config {
    pub path: String,
    pub name: String,
    pub used_count: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OpenVPN3Session {
    pub path: String,
    pub major_code: u32,
    pub minor_code: u32,
    pub status_message: String,
    pub session_created: u64,
}
