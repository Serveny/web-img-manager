use crate::utils::{base64_to_img, resize_image, save_img};
use actix_cors::Cors;
use actix_web::{
    delete, error, get, http::header, middleware::Logger, options, post, web, App, HttpResponse,
    HttpServer, Responder,
};
use sanitize_filename::sanitize;
use serde::Deserialize;
use std::{
    fs::{read_dir, File},
    io::Read,
    path::Path,
};

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

#[get("/list/{config_id}/{chapter_id}")]
async fn get_chapter_img_list(info: web::Path<(String, String)>) -> impl Responder {
    let config_id = sanitize(&info.0);
    let chapter_id = sanitize(&info.1);
    let folder_path = Path::new(IMG_STORAGE_PATH).join(config_id).join(chapter_id);
    let filenames = read_dir(folder_path)
        .ok()
        .map(|entries| {
            entries
                .filter_map(|entry| {
                    entry
                        .ok()
                        .map(|e| e.file_name().to_string_lossy().into_owned())
                })
                .collect()
        })
        .unwrap_or_else(Vec::new);

    HttpResponse::Ok().json(filenames)
}

#[get("/img/{config_id}/{chapter_id}/{filename}")]
async fn get_img(info: web::Path<(String, String, String)>) -> impl Responder {
    let config_id = sanitize(&info.0);
    let chapter_id = sanitize(&info.1);
    let filename = sanitize(&info.2);
    let file_path = Path::new(IMG_STORAGE_PATH)
        .join(config_id)
        .join(chapter_id)
        .join(&filename);

    // Open file
    let Ok(mut file) = File::open(&file_path) else {
        return HttpResponse::NotFound().body("Picture not found");
    };

    // Read file content
    let mut img_content = Vec::new();
    let Ok(_) = file.read_to_end(&mut img_content) else {
        return HttpResponse::NoContent().body("Picture file corrupt");
    };

    // Send file back
    return HttpResponse::Ok()
        .append_header(header::ContentType::jpeg())
        .append_header(header::ContentDisposition::attachment(filename))
        .body(img_content);
}

#[options("/{tail:.*}")]
async fn handle_options() -> impl Responder {
    HttpResponse::Ok()
        .insert_header((header::ACCESS_CONTROL_ALLOW_ORIGIN, "*"))
        .finish()
}

#[derive(Deserialize)]
pub struct UploadRequest {
    config_id: String,
    chapter_id: String,
    image: String,
}

#[post("/upload")]
async fn upload_img(payload: web::Json<UploadRequest>) -> impl Responder {
    let request = payload.0;
    let config_id = sanitize(&request.config_id);
    let chapter_id = sanitize(&request.chapter_id);
    println!("{} - {}", &request.config_id, &request.chapter_id);

    // Read image
    let img = match base64_to_img(request.image.as_str()) {
        Ok(img) => img,
        Err(err_msg) => return HttpResponse::BadRequest().body(err_msg),
    };

    // Process image
    let img = resize_image(img, 4000, 2000);
    let thumb_img = resize_image(img.clone(), 600, 200);

    // Save images
    let img_id = match save_img(img, thumb_img, &config_id, &chapter_id) {
        Ok(id) => id,
        Err(err_msg) => return HttpResponse::InternalServerError().body(err_msg),
    };

    // Send image id back
    HttpResponse::Ok()
        .insert_header((header::ACCESS_CONTROL_ALLOW_ORIGIN, "*"))
        .body(img_id.to_string())
}

#[delete("/delete/{file}")]
async fn delete_img() -> impl Responder {
    HttpResponse::Ok()
}
