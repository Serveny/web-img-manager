use crate::{
    public_messages::ws::{
        ChatMessageEvent, ConnectEvent, ImageProcessedEvent, LobbyDeletedEvent, RoomDeletedEvent,
    },
    utils::ToOutputJsonString,
    ImgId, LobbyId, RoomId, SessionId,
};
use actix::prelude::*;
use serde_json::Error;

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

impl ToOutputJsonString for Connect {
    fn to_output_json_string(&self) -> Result<String, Error> {
        serde_json::to_string(&ConnectEvent {
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

impl ImageUploaded {
    pub fn new(lobby_id: LobbyId, room_id: RoomId, img_id: ImgId) -> Self {
        Self {
            lobby_id,
            room_id,
            img_id,
        }
    }
}

impl ToOutputJsonString for ImageUploaded {
    fn to_output_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&ImageProcessedEvent {
            event: "ImageUploaded",
            room_id: self.room_id,
            img_id: self.img_id,
        })
    }
}

// image was uploaded
#[derive(Message)]
#[rtype(result = "()")]
pub struct ImageDeleted {
    pub lobby_id: LobbyId,
    pub room_id: RoomId,
    pub img_id: ImgId,
}

impl ImageDeleted {
    pub fn new(lobby_id: LobbyId, room_id: RoomId, img_id: ImgId) -> Self {
        Self {
            lobby_id,
            room_id,
            img_id,
        }
    }
}

impl ToOutputJsonString for ImageDeleted {
    fn to_output_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&ImageProcessedEvent {
            event: "ImageDeleted",
            room_id: self.room_id,
            img_id: self.img_id,
        })
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct RoomDeleted {
    pub lobby_id: LobbyId,
    pub room_id: RoomId,
}

impl RoomDeleted {
    pub fn new(lobby_id: LobbyId, room_id: RoomId) -> Self {
        Self { lobby_id, room_id }
    }
}

impl ToOutputJsonString for RoomDeleted {
    fn to_output_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&RoomDeletedEvent {
            event: "RoomDeleted",
            room_id: self.room_id,
        })
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct LobbyDeleted {
    pub lobby_id: LobbyId,
}

impl LobbyDeleted {
    pub fn new(lobby_id: LobbyId) -> Self {
        Self { lobby_id }
    }
}

impl ToOutputJsonString for LobbyDeleted {
    fn to_output_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&LobbyDeletedEvent {
            event: "LobbyDeleted",
        })
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ChatMessage {
    pub lobby_id: LobbyId,
    pub username: String,
    pub msg: String,
}

impl ChatMessage {
    pub fn new(lobby_id: LobbyId, username: String, msg: String) -> Self {
        Self {
            lobby_id,
            username,
            msg,
        }
    }
}

impl ToOutputJsonString for ChatMessage {
    fn to_output_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&ChatMessageEvent {
            event: "ChatMessage",
            username: &self.username,
            msg: &self.msg,
        })
    }
}
