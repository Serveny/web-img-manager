use actix::prelude::*;
use actix_cors::Cors;
use actix_web::{
    error,
    http::header,
    middleware::Logger,
    web::{Data, JsonConfig},
    App, HttpResponse, HttpServer,
};
use api::{
    delete_img, delete_lobby, delete_room, get_img_big, get_img_thumb, get_room_img_list,
    handle_options, upload_img,
};
use config::SERVER;
use uuid::Uuid;
use ws::server::NotifyServer;

mod api;
mod config;
mod utils;
mod ws;

pub type LobbyId = Uuid;
pub type RoomId = Uuid;
pub type ImgId = u32;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    HttpServer::new(|| {
        // json configuration
        let json_cfg = JsonConfig::default()
            .limit(10 * 1024 * 1024) // limit request payload size to 10MB
            .error_handler(|err, _| {
                error::InternalError::from_response(err, HttpResponse::Conflict().into()).into()
            });

        // Live notifications server
        let notify_server = NotifyServer::new().start();

        // Create app
        App::new()
            // -------------
            // config
            // -------------
            .wrap(Logger::default())
            .wrap(cors_cfg())
            .app_data(json_cfg)
            // -------------
            // Notifications
            // -------------
            .app_data(Data::new(notify_server))
            .service(ws::start_connection)
            // -------------
            // API
            // -------------
            .service(get_room_img_list)
            .service(get_img_thumb)
            .service(get_img_big)
            .service(handle_options)
            .service(upload_img)
            .service(delete_room)
            .service(delete_lobby)
            .service(delete_img)
    })
    .bind(SERVER)?
    .run()
    .await
}

fn cors_cfg() -> Cors {
    Cors::default()
        .allowed_methods(vec!["GET", "POST"])
        .allowed_headers(vec![
            header::AUTHORIZATION,
            header::ACCEPT,
            header::CONTENT_TYPE,
            header::CONTENT_LENGTH,
        ])
        .allow_any_origin()
        .supports_credentials()
        .max_age(3600)
}
