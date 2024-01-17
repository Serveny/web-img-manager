use crate::{ImgId, RoomId, SessionId};
use serde::Serialize;
use ts_rs::TS;

#[derive(TS)]
#[ts(export, export_to = "examples/ts-application/src/bindings/")]
#[derive(Serialize)]
pub struct ConnectEvent {
    pub event: &'static str,
    pub session_id: SessionId,
}

#[derive(TS)]
#[ts(export, export_to = "examples/ts-application/src/bindings/")]
#[derive(Serialize)]
pub struct ImageProcessedEvent {
    pub event: &'static str,
    pub room_id: RoomId,
    pub img_id: ImgId,
}

#[derive(TS)]
#[ts(export, export_to = "examples/ts-application/src/bindings/")]
#[derive(Serialize)]
pub struct RoomDeletedEvent {
    pub event: &'static str,
    pub room_id: RoomId,
}

#[derive(TS)]
#[ts(export, export_to = "examples/ts-application/src/bindings/")]
#[derive(Serialize)]
pub struct LobbyDeletedEvent {
    pub event: &'static str,
}
