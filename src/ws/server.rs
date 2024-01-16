use super::messages::{
    ClientActorMessage, Connect, Disconnect, ImageUploaded, ToOutputJsonString, WsMessage,
};
use crate::LobbyId;
use actix::prelude::*;
use log::debug;
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
            debug!("attempting to send message but couldn't find user id.");
        }
    }
}

impl Actor for NotifyServer {
    type Context = Context<Self>;
}

/// Handler for connect message.
impl Handler<Connect> for NotifyServer {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        // create a room if necessary, and then add the id to it
        self.lobbies
            .entry(msg.lobby_id)
            .or_insert_with(HashSet::new)
            .insert(msg.session_id);

        // send to everyone in the room that new uuid just joined
        self.lobbies
            .get(&msg.lobby_id)
            .unwrap()
            .iter()
            .filter(|conn_id| *conn_id.to_owned() != msg.session_id)
            .for_each(|conn_id| {
                self.send_message(&format!("{} just joined!", msg.session_id), conn_id)
            });

        // store the address
        self.sessions.insert(msg.session_id, msg.addr);

        // send self your new uuid
        self.send_message(&format!("your id is {}", msg.session_id), &msg.session_id);
    }
}

/// Handler for Disconnect message.
impl Handler<Disconnect> for NotifyServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        // Remove session from sessions map
        if self.sessions.remove(&msg.session_id).is_none() {
            debug!("Session id to delete not in sessions: {}", msg.session_id);
        }

        // Remove session id from lobby
        let Some(lobby_id) = self
            .lobbies
            .iter_mut()
            .find(|lobby| lobby.1.contains(&msg.session_id))
            .and_then(|lobby| lobby.1.remove(&msg.session_id).then(|| *lobby.0))
        else {
            debug!("Session id to delete not in lobbies: {}", msg.session_id);
            return;
        };

        // Remove lobby if empty
        if self
            .lobbies
            .get(&lobby_id)
            .map(|lobby| lobby.is_empty())
            .unwrap_or(false)
        {
            self.lobbies.remove(&lobby_id);
        }
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
            debug!("Lobby {} not found", msg.lobby_id);
            return;
        };
        let Ok(msg) = msg.to_output_json_string() else {
            debug!("Can't parse event to json");
            return;
        };
        for session_id in lobby {
            if let Some(socket_recipient) = self.sessions.get(session_id) {
                socket_recipient.do_send(WsMessage(msg.clone()));
            } else {
                debug!("Can't socket recipient: {}", session_id);
            }
        }
    }
}
