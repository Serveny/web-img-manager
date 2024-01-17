use crate::{LobbyId, RoomId};
use serde::Deserialize;
use ts_rs::TS;

#[derive(TS)]
#[ts(export)]
#[derive(Deserialize)]
pub struct UploadRequest {
    pub lobby_id: LobbyId,
    pub room_id: RoomId,
    pub image: String,
}