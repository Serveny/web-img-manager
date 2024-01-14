use super::messages::{
    ClientActorMessage, Connect, Disconnect, ImageUploaded, ToOutputJsonString, WsMessage,
};
use crate::LobbyId;
use actix::prelude::*;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

type Socket = Recipient<WsMessage>;
type SessionId = Uuid;

pub struct NotifyServer {
    sessions: HashMap<SessionId, Socket>,
    lobbies: HashMap<LobbyId, HashSet<SessionId>>,
}

impl NotifyServer {
    pub fn new() -> NotifyServer {
        NotifyServer {
            sessions: HashMap::new(),
            lobbies: HashMap::new(),
        }
    }

    pub fn send_message(&self, message: &str, id_to: &Uuid) {
        if let Some(socket_recipient) = self.sessions.get(id_to) {
            let _ = socket_recipient.do_send(WsMessage(message.to_owned()));
        } else {
            println!("attempting to send message but couldn't find user id.");
        }
    }
}

impl Actor for NotifyServer {
    type Context = Context<Self>;
}

/// Handler for Disconnect message.
impl Handler<Disconnect> for NotifyServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        if self.sessions.remove(&msg.id).is_some() {
            self.lobbies
                .get(&msg.room_id)
                .unwrap()
                .iter()
                .filter(|conn_id| *conn_id.to_owned() != msg.id)
                .for_each(|user_id| {
                    self.send_message(&format!("{} disconnected.", &msg.id), user_id)
                });
            if let Some(lobby) = self.lobbies.get_mut(&msg.room_id) {
                if lobby.len() > 1 {
                    lobby.remove(&msg.id);
                } else {
                    //only one in the lobby, remove it entirely
                    self.lobbies.remove(&msg.room_id);
                }
            }
        }
    }
}

impl Handler<Connect> for NotifyServer {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        // create a room if necessary, and then add the id to it
        self.lobbies
            .entry(msg.lobby_id)
            .or_insert_with(HashSet::new)
            .insert(msg.self_id);

        // send to everyone in the room that new uuid just joined
        self.lobbies
            .get(&msg.lobby_id)
            .unwrap()
            .iter()
            .filter(|conn_id| *conn_id.to_owned() != msg.self_id)
            .for_each(|conn_id| {
                self.send_message(&format!("{} just joined!", msg.self_id), conn_id)
            });

        // store the address
        self.sessions.insert(msg.self_id, msg.addr);

        // send self your new uuid
        self.send_message(&format!("your id is {}", msg.self_id), &msg.self_id);
    }
}

impl Handler<ClientActorMessage> for NotifyServer {
    type Result = ();

    fn handle(&mut self, msg: ClientActorMessage, _: &mut Context<Self>) -> Self::Result {
        if msg.msg.starts_with("\\w") {
            if let Some(id_to) = msg.msg.split(' ').collect::<Vec<&str>>().get(1) {
                self.send_message(&msg.msg, &Uuid::parse_str(id_to).unwrap());
            }
        } else {
            self.lobbies
                .get(&msg.room_id)
                .unwrap()
                .iter()
                .for_each(|client| self.send_message(&msg.msg, client));
        }
    }
}

impl Handler<ImageUploaded> for NotifyServer {
    type Result = ();

    fn handle(&mut self, msg: ImageUploaded, _: &mut Context<Self>) -> Self::Result {
        let Some(lobby) = self.lobbies.get(&msg.lobby_id) else {
            return;
        };
        let Ok(msg) = msg.to_output_json_string() else {
            return;
        };
        for session_id in lobby {
            if let Some(socket_recipient) = self.sessions.get(session_id) {
                socket_recipient.do_send(WsMessage(msg.clone()));
            }
        }
    }
}
