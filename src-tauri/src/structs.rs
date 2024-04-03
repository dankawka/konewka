use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ImportConfigPayload {
    pub config_name: String,
    pub single_use: bool,
    pub persistent: bool,
    pub config_file: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LogMessage {
    pub member: String,
    pub first_flag: u32,
    pub second_flag: u32,
    pub message: String,
}
