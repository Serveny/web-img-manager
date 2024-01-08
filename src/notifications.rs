use actix::{Actor, Addr, StreamHandler};
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

pub type Rooms = HashMap<String, Room>;
pub type Room = HashSet<Addr<NotificationsWs>>;

/// Define HTTP actor
pub struct NotificationsWs {
    room_id: String,
}

impl Actor for NotificationsWs {
    type Context = ws::WebsocketContext<Self>;
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for NotificationsWs {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => ctx.text(text),
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}

pub async fn register_client(
    req: HttpRequest,
    stream: web::Payload,
    room_id: String,
) -> Result<HttpResponse, Error> {
    let resp = ws::start(NotificationsWs { room_id }, &req, stream);
    println!("{:?}", resp);
    resp
}

fn get_rooms(req: HttpRequest) -> Option<web::Data<Arc<Mutex<Rooms>>>> {
    req.app_data::<web::Data<Arc<Mutex<Rooms>>>>()
        .map(|r| r.clone())
}

pub fn notify_upload(req: HttpRequest, room_id: String) {
    let Some(rooms) = get_rooms(req) else {
        return;
    };
}
