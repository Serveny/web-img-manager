use actix::Actor;
use actix_cors::Cors;
use actix_web::{error, http::header, middleware::Logger, web, App, HttpResponse, HttpServer};
use config::SERVER;
use notifications::NotificationServer;
use services::{delete_img, get_chapter_img_list, get_img, handle_options, upload_img};

mod config;
mod notifications;
mod services;
mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    HttpServer::new(|| {
        // json configuration
        let json_cfg = web::JsonConfig::default()
            .limit(10 * 1024 * 1024) // limit request payload size to 10MB
            .error_handler(|err, _| {
                error::InternalError::from_response(err, HttpResponse::Conflict().into()).into()
            });

        // Create app
        App::new()
            // config
            .wrap(Logger::default())
            .wrap(cors_cfg())
            .app_data(json_cfg)
            // Notifications
            .app_data(web::Data::new(NotificationServer::new().start()))
            .route("/ws/{room_id}", web::get().to(notifications::join_room))
            // Services
            .service(get_chapter_img_list)
            .service(get_img)
            .service(handle_options)
            .service(upload_img)
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
