use actix_cors::Cors;
use actix_web::{
    delete, get,
    http::header,
    middleware::Logger,
    options, post,
    web::{self},
    App, HttpResponse, HttpServer, Responder,
};
use base64::{engine::general_purpose::STANDARD as Base64, Engine};
use image::{imageops::FilterType, DynamicImage, GenericImageView, ImageFormat};
use sanitize_filename::sanitize;
use serde::Deserialize;
use std::{
    fs::{create_dir_all, File},
    io::Read,
    path::Path,
};
use uuid::Uuid;

const IMG_STORAGE_PATH: &str = ".\\img-test-storage";
const IMG_FORMAT: &str = "jpg";

#[get("/img/{filename}")]
async fn get_img(info: web::Path<(String,)>) -> impl Responder {
    let filename = format!("{}{}{}", sanitize(&info.0), ".", IMG_FORMAT);
    let file_path = Path::new(IMG_STORAGE_PATH).join(&filename);

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
struct UploadRequest {
    config_id: String,
    chapter_id: String,
    image_base64: String,
}

#[post("/upload")]
async fn upload_img(payload: web::Json<UploadRequest>) -> impl Responder {
    let request = payload.0;
    let config_id = sanitize(&request.config_id);
    let chapter_id = sanitize(&request.chapter_id);
    println!("{} - {}", request.config_id, request.chapter_id);

    // Read image
    let Ok(img) = Base64.decode(request.image_base64) else {
        return HttpResponse::BadRequest().body("Can't decode image");
    };
    let Ok(img) = image::load_from_memory(&img) else {
        return HttpResponse::BadRequest().body("Can't load image");
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

fn resize_image(img: DynamicImage, max_width: u32, max_height: u32) -> DynamicImage {
    let (width, height) = img.dimensions();

    if width > max_width || height > max_height {
        return img.resize(max_width, max_height, FilterType::Triangle);
    }
    img
}

fn save_img(
    img: DynamicImage,
    thumb_img: DynamicImage,
    config_id: &str,
    chapter_id: &str,
) -> Result<Uuid, String> {
    // Check storage path
    let storage_path = Path::new(IMG_STORAGE_PATH);
    if !storage_path.exists() {
        return Err(String::from("Storage not found"));
    }

    // Check image folder
    let img_folder_path = storage_path.join(config_id).join(chapter_id);
    if !img_folder_path.exists() && create_dir_all(&img_folder_path).is_err() {
        return Err(String::from("Could not create image folder"));
    }

    // Save images
    let img_id = Uuid::new_v4();

    let img_path = img_folder_path.join(format!("{}.jpg", img_id));
    if let Err(err) = img.save_with_format(img_path, ImageFormat::Jpeg) {
        return Err(err.to_string());
    }

    let thumb_img_path = img_folder_path.join(format!("{}_thumb.jpg", img_id));
    if let Err(err) = thumb_img.save_with_format(thumb_img_path, ImageFormat::Jpeg) {
        return Err(err.to_string());
    }

    Ok(img_id)
}

#[delete("/delete/{file}")]
async fn delete_img() -> impl Responder {
    HttpResponse::Ok()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace")).init();

    HttpServer::new(|| {
        App::new()
            .wrap(
                Cors::default()
                    .allowed_methods(vec!["GET", "POST"])
                    .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
                    .allow_any_origin()
                    .supports_credentials()
                    .max_age(3600),
            )
            .wrap(Logger::default())
            .service(get_img)
            .service(handle_options)
            .service(upload_img)
            .service(delete_img)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
