use crate::{
    config::ServerConfig,
    notification::{
        internal_messages::{ChatMessage, ImageDeleted, ImageUploaded, LobbyDeleted, RoomDeleted},
        server::NotifyServer,
    },
    public_messages::api::{ChatMessageRequest, Success, UploadRequest, UploadResult},
    utils::{
        get_filenames_as_u32, get_foldernames_as_uuid, get_img, img_id_to_filename, read_img,
        resize_image, save_img, ImgType,
    },
    ImgId, LobbyId, RoomId,
};
use actix::prelude::*;
use actix_multipart::form::MultipartForm;
use actix_web::{
    get,
    http::header,
    options, post,
    web::{self, Data, Json},
    HttpResponse, Responder,
};
use log::warn;
use std::{fs, path::Path};

#[get("/list/{lobby_id}")]
pub async fn get_room_list(
    info: web::Path<(LobbyId,)>,
    server_cfg: Data<ServerConfig>,
) -> impl Responder {
    let lobby_id = info.0.to_string();
    let folder_path = Path::new(&server_cfg.images_storage_path).join(lobby_id);
    let filenames = get_foldernames_as_uuid(&folder_path);

    HttpResponse::Ok().json(filenames)
}

#[get("/list/{lobby_id}/{room_id}")]
pub async fn get_room_img_list(
    info: web::Path<(LobbyId, RoomId)>,
    server_cfg: Data<ServerConfig>,
) -> impl Responder {
    let lobby_id = info.0.to_string();
    let room_id = info.1.to_string();
    let folder_path = Path::new(&server_cfg.images_storage_path)
        .join(lobby_id)
        .join(room_id);
    let filenames = get_filenames_as_u32(&folder_path);

    HttpResponse::Ok().json(filenames)
}

#[get("/img/thumb/{lobby_id}/{room_id}/{img_id}")]
pub async fn get_img_thumb(
    info: web::Path<(LobbyId, RoomId, ImgId)>,
    server_cfg: Data<ServerConfig>,
) -> impl Responder {
    get_img(ImgType::Thumb, info, &server_cfg.images_storage_path)
}

#[get("/img/{lobby_id}/{room_id}/{img_id}")]
pub async fn get_img_big(
    info: web::Path<(LobbyId, RoomId, ImgId)>,
    server_cfg: Data<ServerConfig>,
) -> impl Responder {
    get_img(ImgType::Big, info, &server_cfg.images_storage_path)
}

#[options("/{tail:.*}")]
pub async fn handle_options() -> impl Responder {
    HttpResponse::Ok()
        .insert_header((header::ACCESS_CONTROL_ALLOW_ORIGIN, "*"))
        .finish()
}

#[post("/upload/{lobby_id}/{room_id}")]
pub async fn upload_img(
    info: web::Path<(LobbyId, RoomId)>,
    form: MultipartForm<UploadRequest>,
    notify: Data<Addr<NotifyServer>>,
    server_cfg: Data<ServerConfig>,
) -> impl Responder {
    // reject malformed requests
    match form.image.size {
        0 => return HttpResponse::BadRequest().body("Empty image"),
        length if length > server_cfg.max_image_size_byte => {
            return HttpResponse::BadRequest().body(format!(
                "The uploaded file is too large. Maximum size is {} bytes.",
                server_cfg.max_image_size_byte
            ));
        }
        _ => {}
    };

    let lobby_id = info.0;
    let room_id = info.1;

    // Read image
    let img = match read_img(&form.image) {
        Ok(img) => img,
        Err(err_msg) => return HttpResponse::BadRequest().body(err_msg),
    };

    // Process image
    let img = resize_image(img, 4000, 2000);
    let thumb_img = resize_image(img.clone(), 600, 200);

    // Save images
    let img_id = match save_img(
        img,
        thumb_img,
        &lobby_id,
        &room_id,
        &server_cfg.images_storage_path,
    ) {
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
        .json(UploadResult { img_id })
}

#[post("/delete/{lobby_id}")]
pub async fn delete_lobby(
    path: web::Path<(LobbyId,)>,
    notify: Data<Addr<NotifyServer>>,
    server_cfg: Data<ServerConfig>,
) -> impl Responder {
    let folder_path = Path::new(&server_cfg.images_storage_path).join(path.0.to_string());

    // Delete room folder
    if fs::remove_dir_all(&folder_path).is_err() {
        return HttpResponse::InternalServerError()
            .body(format!("Could not delete folder {:?}", folder_path));
    }

    // Notify users
    notify
        .send(LobbyDeleted::new(path.0))
        .await
        .unwrap_or_else(|err| warn!("Can't notify users: {}", err));

    HttpResponse::Ok().json(Success)
}

#[post("/delete/{lobby_id}/{room_id}")]
pub async fn delete_room(
    path: web::Path<(LobbyId, RoomId)>,
    notify: Data<Addr<NotifyServer>>,
    server_cfg: Data<ServerConfig>,
) -> impl Responder {
    let folder_path = Path::new(&server_cfg.images_storage_path)
        .join(path.0.to_string())
        .join(path.1.to_string());

    // Delete room folder
    if fs::remove_dir_all(&folder_path).is_err() {
        return HttpResponse::InternalServerError()
            .body(format!("Could not delete folder {:?}", folder_path));
    }

    // Notify users
    notify
        .send(RoomDeleted::new(path.0, path.1))
        .await
        .unwrap_or_else(|err| warn!("Can't notify users: {}", err));

    HttpResponse::Ok().json(Success)
}

#[post("/delete/{lobby_id}/{room_id}/{file}")]
pub async fn delete_img(
    path: web::Path<(LobbyId, RoomId, ImgId)>,
    notify: Data<Addr<NotifyServer>>,
    server_cfg: Data<ServerConfig>,
) -> impl Responder {
    let room_path = Path::new(&server_cfg.images_storage_path)
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

    HttpResponse::Ok().json(Success)
}

#[post("/chat")]
pub async fn send_chat_message(
    payload: Json<ChatMessageRequest>,
    notify: Data<Addr<NotifyServer>>,
) -> impl Responder {
    let request = payload.0;
    let lobby_id = request.lobby_id;
    let msg = request.msg;

    // Notify users
    notify
        .send(ChatMessage::new(lobby_id, String::from("User"), msg))
        .await
        .unwrap_or_else(|err| warn!("Can't notify users: {}", err));

    HttpResponse::Ok().json(Success)
}
