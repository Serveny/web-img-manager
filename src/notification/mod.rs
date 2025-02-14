use crate::{utils::SESSION_COOKIE_NAME, LobbyId, SessionId};
use actix::prelude::*;
use actix_web::{
    cookie::{Cookie, SameSite},
    get,
    web::{Data, Path, Payload},
    Error, HttpRequest, HttpResponse,
};
use actix_ws::AggregatedMessage;
use futures_util::{
    future::{select, Either},
    StreamExt as _,
};
use internal_messages::{Connect, Disconnect};
use server::NotifyServer;
use std::{
    pin::pin,
    time::{Duration, Instant},
};
use tokio::{sync::mpsc, task::spawn_local, time::interval};
use uuid::Uuid;

pub mod internal_messages;
pub mod server;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(30);

/// Entry point for our websocket route
#[get("/notifications/{lobby_id}")]
pub async fn start_connection(
    req: HttpRequest,
    path: Path<(LobbyId,)>,
    stream: Payload,
    notify_server: Data<Addr<NotifyServer>>,
) -> Result<HttpResponse, Error> {
    let (mut res, session, msg_stream) = actix_ws::handle(&req, stream)?;
    let session_id: SessionId = Uuid::new_v4();

    // spawn websocket handler (and don't await it) so that the response is returned immediately
    spawn_local(chat_ws(
        notify_server,
        session,
        msg_stream,
        session_id,
        path.0,
    ));

    let cookie = Cookie::build(SESSION_COOKIE_NAME, format!("{session_id}; Partitioned"))
        .path("/")
        .http_only(false)
        .secure(true)
        .same_site(SameSite::None)
        .finish();

    res.add_cookie(&cookie)?;

    Ok(res)
}

/// Echo text & binary messages received from the client, respond to ping messages, and monitor
/// connection health to detect network issues and free up resources.
pub async fn chat_ws(
    notify_server: Data<Addr<NotifyServer>>,
    mut session: actix_ws::Session,
    msg_stream: actix_ws::MessageStream,
    session_id: Uuid,
    lobby_id: LobbyId,
) {
    log::info!("connected");

    let mut last_heartbeat = Instant::now();
    let mut interval = interval(HEARTBEAT_INTERVAL);

    let (conn_tx, mut conn_rx) = mpsc::unbounded_channel();

    // unwrap: chat server is not dropped before the HTTP server
    notify_server
        .send(Connect {
            sender: conn_tx,
            lobby_id,
            session_id,
        })
        .await
        .expect("Can connect");

    let msg_stream = msg_stream
        .max_frame_size(128 * 1024)
        .aggregate_continuations()
        .max_continuation_size(2 * 1024 * 1024);

    let mut msg_stream = pin!(msg_stream);

    let close_reason = loop {
        // most of the futures we process need to be stack-pinned to work with select()

        let tick = pin!(interval.tick());
        let msg_rx = pin!(conn_rx.recv());

        // TODO: nested select is pretty gross for readability on the match
        let messages = pin!(select(msg_stream.next(), msg_rx));

        match select(messages, tick).await {
            // messages received from client
            Either::Left((Either::Left((Some(Ok(msg)), _)), _)) => match msg {
                AggregatedMessage::Ping(bytes) => {
                    last_heartbeat = Instant::now();
                    session.pong(&bytes).await.unwrap();
                }
                AggregatedMessage::Pong(_) => last_heartbeat = Instant::now(),
                AggregatedMessage::Text(_text) => log::warn!("unexpected text message"),
                AggregatedMessage::Binary(_bin) => log::warn!("unexpected binary message"),
                AggregatedMessage::Close(reason) => break reason,
            },

            // client WebSocket stream error
            Either::Left((Either::Left((Some(Err(err)), _)), _)) => {
                log::error!("{}", err);
                break None;
            }

            // client WebSocket stream ended
            Either::Left((Either::Left((None, _)), _)) => break None,

            // chat messages received from other room participants
            Either::Left((Either::Right((Some(chat_msg), _)), _)) => {
                session.text(chat_msg).await.unwrap();
            }

            // all connection's message senders were dropped
            Either::Left((Either::Right((None, _)), _)) => unreachable!(
                "all connection message senders were dropped; chat server may have panicked"
            ),

            // heartbeat internal tick
            Either::Right((_inst, _)) => {
                // if no heartbeat ping/pong received recently, close the connection
                if Instant::now().duration_since(last_heartbeat) > CLIENT_TIMEOUT {
                    log::info!(
                        "client has not sent heartbeat in over {CLIENT_TIMEOUT:?}; disconnecting"
                    );
                    break None;
                }

                // send heartbeat ping
                let _ = session.ping(b"").await;
            }
        };
    };

    notify_server
        .send(Disconnect::new(session_id))
        .await
        .expect("Can disconnect");

    // attempt to close connection gracefully
    let _ = session.close(close_reason).await;
}
