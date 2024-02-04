use crate::{ImgId, LobbyId, RoomId};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct UploadRequest {
    pub lobby_id: LobbyId,
    pub room_id: RoomId,
    pub image: String,
}

#[derive(Serialize)]
pub struct UploadResult {
    pub img_id: ImgId,
}

#[derive(Serialize)]
pub struct Success;
