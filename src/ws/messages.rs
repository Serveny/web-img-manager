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

// WsConn sends this to a lobby to say "take me out please"
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub session_id: SessionId,
}

// client sends this to the lobby for the lobby to echo out.
#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientActorMessage {
    pub msg: String,
    pub room_id: RoomId,
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

impl ImageUploaded {
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
        serde_json::to_string(&ImageUploadedOutputEvent {
            event: "ImageUploaded",
            room_id: self.room_id,
            img_id: self.img_id,
        })
    }
}

#[derive(Serialize, Deserialize)]
struct ImageUploadedOutputEvent {
    pub event: &'static str,
    pub room_id: Uuid,
    pub img_id: u32,
}
