use serde::Deserialize;
use ts_rs::TS;

#[derive(Deserialize, TS)]
#[ts(export)]
pub struct ConfirmationResponse {
    pub is_allowed: bool,
    pub error_msg: Option<String>,
}
