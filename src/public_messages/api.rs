use crate::{ImgId, LobbyId, RoomId};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(TS)]
#[ts(export, export_to = "examples/ts-application/src/bindings/")]
#[derive(Deserialize)]
pub struct UploadRequest {
    pub lobby_id: LobbyId,
    pub room_id: RoomId,
    pub image: String,
}

#[derive(TS)]
#[ts(export, export_to = "examples/ts-application/src/bindings/")]
#[derive(Serialize)]
pub struct UploadResult {
    pub img_id: ImgId,
}

#[derive(Serialize)]
pub struct Success;
