use crate::{ImgId, RoomId, SessionId};
use serde::Serialize;

#[derive(Serialize)]
pub struct ConnectEvent {
    pub event: &'static str,
    pub session_id: SessionId,
}

#[derive(Serialize)]
pub struct ImageProcessedEvent {
    pub event: &'static str,
    pub room_id: RoomId,
    pub img_id: ImgId,
}

#[derive(Serialize)]
pub struct RoomDeletedEvent {
    pub event: &'static str,
    pub room_id: RoomId,
}

#[derive(Serialize)]
pub struct LobbyDeletedEvent {
    pub event: &'static str,
}

#[derive(Serialize)]
pub struct ChatMessageEvent<'a> {
    pub event: &'static str,
    pub username: &'a str,
    pub msg: &'a str,
}
