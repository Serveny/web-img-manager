use actix::prelude::*;
use actix_cors::Cors;
use actix_web::{
    error::InternalError,
    http::header,
    middleware::Logger,
    web::{Data, JsonConfig},
    App, HttpResponse, HttpServer,
};
use api::{
    delete_img, delete_lobby, delete_room, get_img_big, get_img_thumb, get_room_img_list,
    get_room_list, handle_options, send_chat_message, test, upload_img,
};
use config::{read_server_config, ServerConfig};
use notification::server::NotifyServer;
use uuid::Uuid;

mod api;
mod config;
mod notification;
mod permission;
mod public_messages;
mod utils;

#[cfg(feature = "openssl")]
mod certificate;

pub type LobbyId = Uuid;
pub type RoomId = u32;
pub type SessionId = Uuid;
pub type ImgId = u32;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    let server_cfg: ServerConfig = read_server_config().unwrap_or_else(|err| {
        println!("{err}. Using hardcoded default config instead");
        ServerConfig::default()
    });
    let server = (server_cfg.url.clone(), server_cfg.port);

    #[cfg(feature = "openssl")]
    let certificate_folder_path = server_cfg.certificate_folder_path.clone();

    // Live notifications server
    let notify_server = Data::new(NotifyServer::new().start());

    let res = HttpServer::new(move || {
        // json configuration
        let json_cfg = JsonConfig::default()
            .limit(10 * 1024 * 1024) // limit request payload size to 10MB
            .error_handler(|err, _| {
                InternalError::from_response(err, HttpResponse::BadRequest().into()).into()
            });

        // Create app
        App::new()
            // -------------
            // config
            // -------------
            .wrap(Logger::default())
            .wrap(cors_cfg())
            .app_data(json_cfg)
            .app_data(Data::new(server_cfg.clone()))
            // -------------
            // Notifications
            // -------------
            .app_data(notify_server.clone())
            .service(notification::start_connection)
            // -------------
            // API
            // -------------
            .service(get_room_list)
            .service(get_room_img_list)
            .service(get_img_thumb)
            .service(get_img_big)
            .service(handle_options)
            .service(upload_img)
            .service(delete_room)
            .service(delete_lobby)
            .service(delete_img)
            .service(send_chat_message)
            .service(test)
    });

    #[cfg(feature = "openssl")]
    let res = match certificate_folder_path {
        Some(path) => res.bind_openssl(
            &server,
            certificate::load_ssl_certificate(&path)
                .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))?,
        )?,

        None => res.bind(&server)?,
    };

    #[cfg(not(feature = "openssl"))]
    let res = res.bind(&server)?;

    println!("Server listening to {}:{}", server.0, server.1);
    res.run().await
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
