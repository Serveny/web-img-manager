use crate::{ImgId, LobbyId, RoomId, SessionId};
use actix::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Error;
use uuid::Uuid;

pub trait ToOutputJsonString {
    fn to_output_json_string(&self) -> Result<String, Error>;
}

// WsConn responds to this to pipe it through to the actual client
#[derive(Message)]
#[rtype(result = "()")]
pub struct WsMessage(pub String);

// WsConn sends this to the lobby to say "put me in please"
#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub addr: Recipient<WsMessage>,
    pub lobby_id: LobbyId,
    pub session_id: SessionId,
}

#[derive(Serialize, Deserialize)]
struct ConnectOutputEvent {
    pub event: &'static str,
    pub session_id: Uuid,
}

impl ToOutputJsonString for Connect {
    fn to_output_json_string(&self) -> Result<String, Error> {
        serde_json::to_string(&ConnectOutputEvent {
            event: "Connected",
            session_id: self.session_id,
        })
    }
}

// WsConn sends this to a lobby to say "take me out please"
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub session_id: SessionId,
}

// image was uploaded
#[derive(Message)]
#[rtype(result = "()")]
pub struct ImageUploaded {
    pub lobby_id: LobbyId,
    pub room_id: RoomId,
    pub img_id: ImgId,
}

// image was uploaded
#[derive(Message)]
#[rtype(result = "()")]
pub struct ImageDeleted {
    pub lobby_id: LobbyId,
    pub room_id: RoomId,
    pub img_id: ImgId,
}

#[derive(Serialize, Deserialize)]
struct ImageProcessedOutputEvent {
    pub event: &'static str,
    pub room_id: Uuid,
    pub img_id: u32,
}

impl ImageUploaded {
    pub fn new(lobby_id: Uuid, room_id: Uuid, img_id: u32) -> Self {
        Self {
            lobby_id,
            room_id,
            img_id,
        }
    }
}

impl ImageDeleted {
    pub fn new(lobby_id: Uuid, room_id: Uuid, img_id: u32) -> Self {
        Self {
            lobby_id,
            room_id,
            img_id,
        }
    }
}
impl ToOutputJsonString for ImageUploaded {
    fn to_output_json_string(&self) -> Result<String, Error> {
        serde_json::to_string(&ImageProcessedOutputEvent {
            event: "ImageUploaded",
            room_id: self.room_id,
            img_id: self.img_id,
        })
    }
}

impl ToOutputJsonString for ImageDeleted {
    fn to_output_json_string(&self) -> Result<String, Error> {
        serde_json::to_string(&ImageProcessedOutputEvent {
            event: "ImageDeleted",
            room_id: self.room_id,
            img_id: self.img_id,
        })
    }
}
