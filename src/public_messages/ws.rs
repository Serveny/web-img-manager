use crate::{ImgId, RoomId, SessionId};
use serde::Serialize;
use ts_rs::TS;

#[derive(TS)]
#[ts(export)]
#[derive(Serialize)]
pub struct ConnectEvent {
    pub event: &'static str,
    pub session_id: SessionId,
}

#[derive(TS)]
#[ts(export)]
#[derive(Serialize)]
pub struct ImageProcessedEvent {
    pub event: &'static str,
    pub room_id: RoomId,
    pub img_id: ImgId,
}
