use crate::{ImgId, LobbyId};
use actix_multipart::form::{tempfile::TempFile, MultipartForm};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, MultipartForm, TS)]
#[ts(export)]
pub struct UploadRequest {
    #[multipart]
    #[ts(type = "File")]
    pub image: TempFile,
}

#[derive(Serialize, TS)]
#[ts(export)]
pub struct UploadResult {
    pub img_id: ImgId,
}

#[derive(Serialize, TS)]
#[ts(export)]
pub struct Success;

#[derive(Deserialize, TS)]
#[ts(export)]
pub struct ChatMessageRequest {
    pub lobby_id: LobbyId,
    pub msg: String,
}
