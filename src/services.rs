use crate::{
    config::IMG_STORAGE_PATH,
    notifications::{NotificationMessage, NotificationServer},
    utils::{base64_to_img, get_filenames, resize_image, save_img},
};
use actix_web::{delete, get, http::header, options, post, web, HttpResponse, Responder};
use sanitize_filename::sanitize;
use serde::Deserialize;
use std::{fs::File, io::Read, path::Path};

#[get("/list/{room_id}/{chapter_id}")]
pub async fn get_chapter_img_list(info: web::Path<(String, String)>) -> impl Responder {
    let room_id = sanitize(&info.0);
    let chapter_id = sanitize(&info.1);
    let folder_path = Path::new(IMG_STORAGE_PATH).join(room_id).join(chapter_id);
    let filenames = get_filenames(&folder_path);

    HttpResponse::Ok().json(filenames)
}

#[get("/img/{room_id}/{chapter_id}/{filename}")]
pub async fn get_img(info: web::Path<(String, String, String)>) -> impl Responder {
    let room_id = sanitize(&info.0);
    let chapter_id = sanitize(&info.1);
    let filename = sanitize(&info.2);
    let file_path = Path::new(IMG_STORAGE_PATH)
        .join(room_id)
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
pub async fn handle_options() -> impl Responder {
    HttpResponse::Ok()
        .insert_header((header::ACCESS_CONTROL_ALLOW_ORIGIN, "*"))
        .finish()
}

#[derive(Deserialize)]
pub struct UploadRequest {
    room_id: String,
    chapter_id: String,
    image: String,
}

#[post("/upload")]
pub async fn upload_img(
    payload: web::Json<UploadRequest>,
    notification: web::Data<NotificationServer>,
) -> impl Responder {
    let request = payload.0;
    let room_id = sanitize(&request.room_id);
    let chapter_id = sanitize(&request.chapter_id);

    // Read image
    let img = match base64_to_img(request.image.as_str()) {
        Ok(img) => img,
        Err(err_msg) => return HttpResponse::BadRequest().body(err_msg),
    };

    // Process image
    let img = resize_image(img, 4000, 2000);
    let thumb_img = resize_image(img.clone(), 600, 200);

    // Save images
    let img_id = match save_img(img, thumb_img, &room_id, &chapter_id) {
        Ok(id) => id,
        Err(err_msg) => return HttpResponse::InternalServerError().body(err_msg),
    };

    // Notify users about image upload
    notification.send_message(
        &room_id,
        NotificationMessage::ImageUpload { chapter_id, img_id },
    );

    // Send image id back
    HttpResponse::Ok()
        .insert_header((header::ACCESS_CONTROL_ALLOW_ORIGIN, "*"))
        .body(img_id.to_string())
}

#[delete("/delete/{file}")]
pub async fn delete_img() -> impl Responder {
    HttpResponse::Ok()
}
