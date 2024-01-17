use crate::{
    config::IMG_STORAGE_PATH,
    notification::{
        internal_messages::{ImageDeleted, ImageUploaded},
        server::NotifyServer,
    },
    public_messages::api::UploadRequest,
    utils::{
        base64_to_img, delete_folder, get_filenames_as_u32, get_img, img_id_to_filename,
        resize_image, save_img, ImgType,
    },
    ImgId, LobbyId, RoomId,
};
use actix::prelude::*;
use actix_web::{
    get,
    http::header,
    options, post,
    web::{self, Data, Json},
    HttpResponse, Responder,
};
use log::warn;
use std::{
    fs::{self},
    path::Path,
};

#[get("/list/{lobby_id}/{room_id}")]
pub async fn get_room_img_list(info: web::Path<(LobbyId, RoomId)>) -> impl Responder {
    let lobby_id = info.0.to_string();
    let room_id = info.1.to_string();
    let folder_path = Path::new(IMG_STORAGE_PATH).join(lobby_id).join(room_id);
    let filenames = get_filenames_as_u32(&folder_path);

    HttpResponse::Ok().json(filenames)
}

#[get("/img/thumb/{lobby_id}/{room_id}/{img_id}")]
pub async fn get_img_thumb(info: web::Path<(LobbyId, RoomId, ImgId)>) -> impl Responder {
    get_img(ImgType::Thumb, info)
}

#[get("/img/{lobby_id}/{room_id}/{img_id}")]
pub async fn get_img_big(info: web::Path<(LobbyId, RoomId, ImgId)>) -> impl Responder {
    get_img(ImgType::Big, info)
}

#[options("/{tail:.*}")]
pub async fn handle_options() -> impl Responder {
    HttpResponse::Ok()
        .insert_header((header::ACCESS_CONTROL_ALLOW_ORIGIN, "*"))
        .finish()
}

#[post("/upload")]
pub async fn upload_img(
    payload: Json<UploadRequest>,
    notify: Data<Addr<NotifyServer>>,
) -> impl Responder {
    let request = payload.0;
    let lobby_id = request.lobby_id;
    let room_id = request.room_id;

    // Read image
    let img = match base64_to_img(request.image.as_str()) {
        Ok(img) => img,
        Err(err_msg) => return HttpResponse::BadRequest().body(err_msg),
    };

    // Process image
    let img = resize_image(img, 4000, 2000);
    let thumb_img = resize_image(img.clone(), 600, 200);

    // Save images
    let img_id = match save_img(img, thumb_img, &lobby_id, &room_id) {
        Ok(id) => id,
        Err(err_msg) => return HttpResponse::InternalServerError().body(err_msg),
    };

    // Notify users
    notify
        .send(ImageUploaded::new(lobby_id, room_id, img_id))
        .await
        .unwrap_or_else(|err| warn!("Can't notify users: {}", err));

    // Send image id back
    HttpResponse::Ok()
        .insert_header((header::ACCESS_CONTROL_ALLOW_ORIGIN, "*"))
        .body(img_id.to_string())
}

#[post("/delete/{lobby_id}")]
pub async fn delete_lobby(path: web::Path<(LobbyId,)>) -> impl Responder {
    let folder_path = Path::new(IMG_STORAGE_PATH).join(path.0.to_string());
    delete_folder(&folder_path)
}

#[post("/delete/{lobby_id}/{room_id}")]
pub async fn delete_room(path: web::Path<(LobbyId, RoomId)>) -> impl Responder {
    let folder_path = Path::new(IMG_STORAGE_PATH)
        .join(path.0.to_string())
        .join(path.1.to_string());
    delete_folder(&folder_path)
}

#[post("/delete/{lobby_id}/{room_id}/{file}")]
pub async fn delete_img(
    path: web::Path<(LobbyId, RoomId, ImgId)>,
    notify: Data<Addr<NotifyServer>>,
) -> impl Responder {
    let room_path = Path::new(IMG_STORAGE_PATH)
        .join(path.0.to_string())
        .join(path.1.to_string());
    let filename = img_id_to_filename(path.2);

    // Delete big image
    let img_path = room_path.join(&filename);
    fs::remove_file(img_path).unwrap_or_default();

    // Delete thumb image
    let thumb_path = room_path.join("thumb").join(filename);
    fs::remove_file(thumb_path).unwrap_or_default();

    // Notify users
    notify
        .send(ImageDeleted::new(path.0, path.1, path.2))
        .await
        .unwrap_or_else(|err| warn!("Can't notify users: {}", err));

    HttpResponse::Ok().finish()
}
