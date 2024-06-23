use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ConfirmationResponse {
    pub is_allowed: bool,
    pub error_msg: Option<String>,
}
