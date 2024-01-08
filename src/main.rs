use actix_cors::Cors;
use actix_web::{error, http::header, middleware::Logger, web, App, HttpResponse, HttpServer};
use config::SERVER;
use notifications::Rooms;
use services::{delete_img, get_chapter_img_list, get_img, handle_options, upload_img};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

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

        // HashMap to save all notification subscriber
        let rooms: Arc<Mutex<Rooms>> = Arc::new(Mutex::new(HashMap::new()));

        // Create app
        App::new()
            // config
            .wrap(Logger::default())
            .wrap(
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
                    .max_age(3600),
            )
            .app_data(json_cfg)
            // Notifications
            .app_data(web::Data::new(rooms.clone()))
            .route("/ws/", web::get().to(notifications::register_client))
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
