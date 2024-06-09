use crate::{ImgId, LobbyId};
use actix_multipart::form::{tempfile::TempFile, MultipartForm};
use serde::{Deserialize, Serialize};

#[derive(Debug, MultipartForm)]
pub struct UploadRequest {
    #[multipart]
    pub image: TempFile,
}

#[derive(Serialize)]
pub struct UploadResult {
    pub img_id: ImgId,
}

#[derive(Serialize)]
pub struct Success;

#[derive(Deserialize)]
pub struct ChatMessageRequest {
    pub lobby_id: LobbyId,
    pub msg: String,
}
