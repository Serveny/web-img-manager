use super::internal_messages::{Connect, Disconnect, ImageDeleted, ImageUploaded, WsMessage};
use crate::{utils::ToOutputJsonString, LobbyId};
use actix::prelude::*;
use log::{debug, warn};
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
        debug!("Server instance created");
        NotifyServer {
            sessions: HashMap::new(),
            lobbies: HashMap::new(),
        }
    }

    fn send_msg(&self, session_id: &SessionId, msg: &impl ToOutputJsonString) {
        match msg.to_output_json_string() {
            Ok(msg_json) => self.send_string(session_id, &msg_json),
            Err(err) => {
                warn!("Can't parse event to json: {}", err);
                return;
            }
        };
    }

    fn send_msg_to_lobby(&self, lobby_id: &LobbyId, msg: &str) {
        let Some(lobby) = self.lobbies.get(lobby_id) else {
            warn!("Lobby {} not found", lobby_id);
            return;
        };
        for session_id in lobby {
            self.send_string(session_id, msg);
        }
    }

    fn send_string(&self, session_id: &SessionId, msg: &str) {
        match self.sessions.get(session_id) {
            Some(socket_recipient) => socket_recipient.do_send(WsMessage(msg.to_string())),
            None => warn!("Can't find socket recipient: {}", session_id),
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

        debug!("Lobbies: {:?}", self.lobbies);

        // send self your new uuid
        self.send_msg(&msg.session_id, &msg);

        // store the address
        self.sessions.insert(msg.session_id, msg.addr);
    }
}

/// Handler for Disconnect message.
impl Handler<Disconnect> for NotifyServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        // Remove session from sessions map
        if self.sessions.remove(&msg.session_id).is_none() {
            warn!("Session id to delete not in sessions: {}", msg.session_id);
        }

        // Remove session id from lobby
        let Some(lobby_id) = self
            .lobbies
            .iter_mut()
            .find(|lobby| lobby.1.contains(&msg.session_id))
            .and_then(|lobby| lobby.1.remove(&msg.session_id).then(|| *lobby.0))
        else {
            warn!("Session id to delete not in lobbies: {}", msg.session_id);
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

impl Handler<ImageUploaded> for NotifyServer {
    type Result = ();

    fn handle(&mut self, msg: ImageUploaded, _: &mut Context<Self>) -> Self::Result {
        let Ok(msg_json) = msg.to_output_json_string() else {
            warn!("Can't parse image uploaded event to json");
            return;
        };
        self.send_msg_to_lobby(&msg.lobby_id, &msg_json);
    }
}

impl Handler<ImageDeleted> for NotifyServer {
    type Result = ();

    fn handle(&mut self, msg: ImageDeleted, _: &mut Context<Self>) -> Self::Result {
        let Ok(msg_json) = msg.to_output_json_string() else {
            warn!("Can't parse image deleted event to json");
            return;
        };
        self.send_msg_to_lobby(&msg.lobby_id, &msg_json);
    }
}