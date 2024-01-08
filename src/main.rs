use actix_cors::Cors;
use actix_web::{error, http::header, middleware::Logger, web, App, HttpResponse, HttpServer};
use services::{delete_img, get_chapter_img_list, get_img, handle_options, upload_img};

mod services;
mod utils;

pub const IMG_STORAGE_PATH: &str = "./img-test-storage";

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    HttpServer::new(|| {
        // custom `Json` extractor configuration
        let json_cfg = web::JsonConfig::default()
            // limit request payload size
            .limit(10 * 1024 * 1024) // 10MB
            // use custom error handler
            .error_handler(|err, _| {
                error::InternalError::from_response(err, HttpResponse::Conflict().into()).into()
            });
        App::new()
            .app_data(json_cfg)
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
            .wrap(Logger::default())
            .service(get_chapter_img_list)
            .service(get_img)
            .service(handle_options)
            .service(upload_img)
            .service(delete_img)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
