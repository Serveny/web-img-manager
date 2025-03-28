use actix::prelude::*;
use actix_web::{
    error::InternalError,
    middleware::Logger,
    web::{Data, JsonConfig},
    App, HttpResponse, HttpServer,
};
use api::{
    delete_img, delete_lobby, delete_room, get_img_big, get_img_thumb, get_room_img_list,
    get_room_list, handle_options, send_chat_message, test, upload_img,
};
use check::ImgChecker;
use config::{cors_cfg, read_server_config, ServerConfig};
use log::{error, info};
use notification::server::NotifyServer;
use uuid::Uuid;

mod api;
mod check;
mod config;
mod img;
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
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let server_cfg: Data<ServerConfig> = Data::new(read_server_config().unwrap_or_else(|err| {
        error!("{err}. Using hardcoded default config instead");
        ServerConfig::default()
    }));
    let server = (server_cfg.url.clone(), server_cfg.port);

    #[cfg(feature = "openssl")]
    let cert_pem_path = server_cfg.cert_pem_path.clone();

    #[cfg(feature = "openssl")]
    let key_pem_path = server_cfg.key_pem_path.clone();

    // Live notifications server
    let notify_server = Data::new(NotifyServer::new().start());
    let img_checker = Data::new(ImgChecker::new(notify_server.clone(), server_cfg.clone()).start());

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
            .app_data(server_cfg.clone())
            // -------------
            // Notifications
            // -------------
            .app_data(notify_server.clone())
            .service(notification::start_connection)
            // -------------
            // After upload check
            // -------------
            .app_data(img_checker.clone())
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
    let res = match (cert_pem_path, key_pem_path) {
        (Some(cert_path), Some(key_path)) => {
            info!("SSL active");
            res.bind_openssl(
                &server,
                certificate::load_ssl_certificate(&cert_path, &key_path)
                    .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))?,
            )
        }
        (None, None) => Ok(res.bind(&server)?),
        _ => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Wrong configuration: cert.pem and key.pem needed.",
            ))
        }
    }?;

    #[cfg(not(feature = "openssl"))]
    let res = res.bind(&server)?;

    info!("Server listening to {}:{}", server.0, server.1);
    res.run().await
}
