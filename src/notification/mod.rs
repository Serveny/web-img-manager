use crate::{LobbyId, SessionId};
use actix::prelude::*;
use actix_web::{
    get,
    web::{Data, Path, Payload},
    Error, HttpRequest, HttpResponse,
};
use actix_web_actors::ws::{self};
use internal_messages::{Connect, Disconnect, WsMessage};
use log::debug;
use server::NotifyServer;
use std::time::{Duration, Instant};
use uuid::Uuid;

pub mod internal_messages;
pub mod server;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(30);

/// Entry point for our websocket route
#[get("/ws/{lobby_id}")]
pub async fn start_connection(
    req: HttpRequest,
    path: Path<(LobbyId,)>,
    stream: Payload,
    srv: Data<Addr<NotifyServer>>,
) -> Result<HttpResponse, Error> {
    let ws = WsConn::new(path.0, srv.get_ref().clone());
    ws::start(ws, &req, stream)
}

pub struct WsConn {
    session_id: SessionId,
    lobby_id: LobbyId,
    hb: Instant,
    lobby_addr: Addr<NotifyServer>,
}

impl WsConn {
    pub fn new(lobby_id: LobbyId, lobby: Addr<NotifyServer>) -> WsConn {
        WsConn {
            session_id: Uuid::new_v4(),
            lobby_id,
            hb: Instant::now(),
            lobby_addr: lobby,
        }
    }

    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                println!("Disconnecting failed heartbeat");
                ctx.stop();
                return;
            }

            ctx.ping(b"PING");
        });
    }
}

impl Actor for WsConn {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);

        let addr = ctx.address();
        self.lobby_addr
            .send(Connect {
                addr: addr.recipient(),
                lobby_id: self.lobby_id,
                session_id: self.session_id,
            })
            .into_actor(self)
            .then(|res, _, ctx| {
                match res {
                    Ok(_res) => (),
                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        self.lobby_addr.do_send(Disconnect {
            session_id: self.session_id,
        });
        Running::Stop
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsConn {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            Ok(ws::Message::Continuation(_)) => {
                ctx.stop();
            }
            Ok(ws::Message::Nop) => (),
            Ok(ws::Message::Text(s)) => debug!("Text send: {}", s),
            Err(e) => println!("{}", e),
        }
    }
}

impl Handler<WsMessage> for WsConn {
    type Result = ();

    fn handle(&mut self, msg: WsMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}
