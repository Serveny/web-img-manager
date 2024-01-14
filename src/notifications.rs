use actix::prelude::*;
use actix_web::{
    web::{self, Data},
    Error, HttpRequest, HttpResponse,
};
use actix_web_actors::ws;
use rand::{rngs::ThreadRng, Rng};
use std::collections::{HashMap, HashSet};

#[derive(actix::Message, Clone)]
#[rtype(result = "()")]
pub enum NotificationMessage {
    ImageUpload { chapter_id: String, img_id: u32 },
}

#[derive(Debug)]
pub struct NotificationServer {
    sessions: HashMap<usize, Recipient<NotificationMessage>>,
    rooms: HashMap<String, HashSet<usize>>,
    rng: ThreadRng,
}

impl NotificationServer {
    pub fn new() -> Self {
        NotificationServer {
            sessions: HashMap::new(),
            rooms: HashMap::new(),
            rng: rand::thread_rng(),
        }
    }

    /// Send message to all users in the room
    pub fn send_message(&self, room_id: &str, msg: NotificationMessage) {
        let Some(room) = self.rooms.get(room_id) else {
            return;
        };
        for id in room {
            if let Some(addr) = self.sessions.get(id) {
                addr.do_send(msg.clone());
            }
        }
    }
}

/// Make actor from `NotificationServer`
impl Actor for NotificationServer {
    /// We are going to use simple Context, we just need ability to communicate
    /// with other actors.
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
    pub addr: Recipient<NotificationMessage>,
    pub room_id: String,
}

/// Handler for Connect message.
///
/// Register new session and assign unique id to this session
impl Handler<Connect> for NotificationServer {
    type Result = usize;

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        println!("Someone joined");
        // register session with random id
        let id = self.rng.gen::<usize>();
        self.sessions.insert(id, msg.addr);

        // auto join session to main room
        self.rooms.entry(msg.room_id).or_default().insert(id);

        // send id back
        id
    }
}

/// Session is disconnected
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: usize,
}

/// Handler for Disconnect message.
impl Handler<Disconnect> for NotificationServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        println!("Someone disconnected");

        // remove address
        if self.sessions.remove(&msg.id).is_some() {
            // remove session from all rooms
            for sessions in &mut self.rooms.values_mut() {
                sessions.remove(&msg.id);
            }
        }
    }
}

#[derive(Debug)]
pub struct NotificationSession {
    /// unique session id
    pub id: usize,

    /// joined room
    pub room_id: String,

    pub srv: Data<Addr<NotificationServer>>,
}

impl Actor for NotificationSession {
    type Context = ws::WebsocketContext<Self>;
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for NotificationSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        println!("Session stated");
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => ctx.text(text),
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}

/// Entry point for our websocket route
pub async fn start_connection(
    req: HttpRequest,
    path: web::Path<(String,)>,
    stream: web::Payload,
    srv: Data<Addr<NotificationServer>>,
) -> Result<HttpResponse, Error> {
    ws::start(
        NotificationSession {
            id: 0,
            room_id: path.0.clone(),
            srv: srv.clone(),
        },
        &req,
        stream,
    )
}
