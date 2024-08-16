use crate::{ImgId, RoomId, SessionId};
use serde::Serialize;
use ts_rs::TS;

#[derive(Serialize, TS)]
#[ts(export)]
pub struct ConnectEvent {
    pub event: &'static str,
    pub session_id: SessionId,
}

#[derive(Serialize, TS)]
#[ts(export)]
pub struct ImageProcessedEvent {
    pub event: &'static str,
    pub room_id: RoomId,
    pub img_id: ImgId,
}

#[derive(Serialize, TS)]
#[ts(export)]
pub struct RoomDeletedEvent {
    pub event: &'static str,
    pub room_id: RoomId,
}

#[derive(Serialize, TS)]
#[ts(export)]
pub struct LobbyDeletedEvent {
    pub event: &'static str,
}

#[derive(Serialize, TS)]
#[ts(export)]
pub struct ChatMessageEvent<'a> {
    pub event: &'static str,
    pub username: &'a str,
    pub msg: &'a str,
}

#[derive(Serialize, TS)]
#[ts(export)]
pub struct SystemNotificationEvent<'a> {
    pub event: &'static str,
    pub msg: &'a str,
    pub msg_type: &'a str,
}
