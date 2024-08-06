use crate::{
    config::ServerConfig,
    notification::{
        internal_messages::{ChatMessage, ImageDeleted, ImageUploaded, LobbyDeleted, RoomDeleted},
        server::NotifyServer,
    },
    permission::check,
    public_messages::api::{ChatMessageRequest, Success, UploadRequest, UploadResult},
    utils::{
        check_image, delete_img_files, get_filenames_as_img_id, get_foldernames_as_uuid, get_img,
        read_img, resize_image, save_img, ImgType, SaveImageResult,
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
    HttpRequest, HttpResponse, Responder,
};
use log::{debug, warn};
use std::{fs, path::Path};

#[get("/list/{lobby_id}")]
pub async fn get_room_list(
    info: web::Path<(LobbyId,)>,
    cfg: Data<ServerConfig>,
    req: HttpRequest,
) -> impl Responder {
    let lobby_id = info.0.to_string();

    // check permission
    if let Some(err) = check(&cfg.permissions.get_room_list, &req, &info.into_inner()).await {
        return err;
    }

    let folder_path = Path::new(&cfg.images_storage_path).join(lobby_id);
    let filenames = get_foldernames_as_uuid(&folder_path);

    HttpResponse::Ok().json(filenames)
}

#[get("/list/{lobby_id}/{room_id}")]
pub async fn get_room_img_list(
    info: web::Path<(LobbyId, RoomId)>,
    cfg: Data<ServerConfig>,
    req: HttpRequest,
) -> impl Responder {
    let lobby_id = info.0.to_string();
    let room_id = info.1.to_string();

    // check permission
    if let Some(err) = check(&cfg.permissions.get_room_img_list, &req, &info.into_inner()).await {
        return err;
    }

    let folder_path = Path::new(&cfg.images_storage_path)
        .join(lobby_id)
        .join(room_id);

    HttpResponse::Ok().json(get_filenames_as_img_id(&folder_path).unwrap_or_default())
}

#[get("/img/thumb/{lobby_id}/{room_id}/{img_id}")]
pub async fn get_img_thumb(
    info: web::Path<(LobbyId, RoomId, ImgId)>,
    cfg: Data<ServerConfig>,
    req: HttpRequest,
) -> impl Responder {
    let params = info.into_inner();

    // check permission
    if let Some(err) = check(&cfg.permissions.get_img_thumb, &req, &params).await {
        return err;
    }
    get_img(ImgType::Thumb, &params, &cfg.images_storage_path)
}

#[get("/img/{lobby_id}/{room_id}/{img_id}")]
pub async fn get_img_big(
    info: web::Path<(LobbyId, RoomId, ImgId)>,
    cfg: Data<ServerConfig>,
    req: HttpRequest,
) -> impl Responder {
    let params = info.into_inner();

    // check permission
    if let Some(err) = check(&cfg.permissions.get_img_big, &req, &params).await {
        return err;
    }

    get_img(ImgType::Big, &params, &cfg.images_storage_path)
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
    cfg: Data<ServerConfig>,
    req: HttpRequest,
) -> impl Responder {
    let lobby_id = info.0;
    let room_id = info.1;

    // check permission
    if let Some(err) = check(&cfg.permissions.upload_img, &req, &info.into_inner()).await {
        return err;
    }

    // reject malformed requests
    match form.image.size {
        0 => return HttpResponse::BadRequest().body("Empty image"),
        length if length > cfg.max_image_size_byte => {
            return HttpResponse::BadRequest().body(format!(
                "The uploaded file is too large. Maximum size is {} bytes.",
                cfg.max_image_size_byte
            ));
        }
        _ => {}
    };

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
        &img,
        &thumb_img,
        &lobby_id,
        &room_id,
        &cfg.images_storage_path,
    ) {
        SaveImageResult::Ok(id) => id,
        SaveImageResult::ImageAlreadyExists(img_id) => {
            return HttpResponse::Ok()
                .insert_header((header::ACCESS_CONTROL_ALLOW_ORIGIN, "*"))
                .json(UploadResult { img_id })
        }
        SaveImageResult::Err(err_msg) => return HttpResponse::InternalServerError().body(err_msg),
    };

    // After upload check (TODO: Make this check async after response)
    if let Some(check) = &cfg.after_upload_check {
        match check_image(&check.url, thumb_img, img_id).await {
            Ok(is_allowed) if !is_allowed => {
                debug!("Img {img_id} not allowed");
                delete_img_files((lobby_id, room_id, img_id), &cfg.images_storage_path);
                return HttpResponse::Forbidden().body("NSFW image detected");
            }
            Ok(_) => debug!("Img {img_id} allowed"),
            Err(err) => debug!("{err}"),
        };
    }

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
    cfg: Data<ServerConfig>,
    req: HttpRequest,
) -> impl Responder {
    let lobby_id = path.0;

    // check permission
    if let Some(err) = check(&cfg.permissions.delete_lobby, &req, &path.into_inner()).await {
        return err;
    }

    let folder_path = Path::new(&cfg.images_storage_path).join(lobby_id.to_string());

    // Delete room folder
    if fs::remove_dir_all(&folder_path).is_err() {
        return HttpResponse::InternalServerError()
            .body(format!("Could not delete folder {:?}", folder_path));
    }

    // Notify users
    notify
        .send(LobbyDeleted::new(lobby_id))
        .await
        .unwrap_or_else(|err| warn!("Can't notify users: {}", err));

    HttpResponse::Ok().json(Success)
}

#[post("/delete/{lobby_id}/{room_id}")]
pub async fn delete_room(
    path: web::Path<(LobbyId, RoomId)>,
    notify: Data<Addr<NotifyServer>>,
    cfg: Data<ServerConfig>,
    req: HttpRequest,
) -> impl Responder {
    let lobby_id = path.0;
    let room_id = path.1;

    // check permission
    if let Some(err) = check(&cfg.permissions.delete_room, &req, &path.into_inner()).await {
        return err;
    }

    let folder_path = Path::new(&cfg.images_storage_path)
        .join(lobby_id.to_string())
        .join(room_id.to_string());

    // Delete room folder
    if fs::remove_dir_all(&folder_path).is_err() {
        return HttpResponse::InternalServerError()
            .body(format!("Could not delete folder {:?}", folder_path));
    }

    // Notify users
    notify
        .send(RoomDeleted::new(lobby_id, room_id))
        .await
        .unwrap_or_else(|err| warn!("Can't notify users: {}", err));

    HttpResponse::Ok().json(Success)
}

#[post("/delete/{lobby_id}/{room_id}/{file}")]
pub async fn delete_img(
    path: web::Path<(LobbyId, RoomId, ImgId)>,
    notify: Data<Addr<NotifyServer>>,
    cfg: Data<ServerConfig>,
    req: HttpRequest,
) -> impl Responder {
    let lobby_id = path.0;
    let room_id = path.1;
    let img_id = path.2;

    // check permission
    if let Some(err) = check(&cfg.permissions.delete_img, &req, &path.into_inner()).await {
        return err;
    }

    delete_img_files((lobby_id, room_id, img_id), &cfg.images_storage_path);

    // Notify users
    notify
        .send(ImageDeleted::new(lobby_id, room_id, img_id))
        .await
        .unwrap_or_else(|err| warn!("Can't notify users: {}", err));

    HttpResponse::Ok().json(Success)
}

#[post("/chat")]
pub async fn send_chat_message(
    payload: Json<ChatMessageRequest>,
    notify: Data<Addr<NotifyServer>>,
    cfg: Data<ServerConfig>,
    req: HttpRequest,
) -> impl Responder {
    let request = payload.0;
    let lobby_id = request.lobby_id;
    let msg = request.msg;

    // check permission
    if let Some(err) = check(&cfg.permissions.send_chat_message, &req, &(lobby_id,)).await {
        return err;
    }

    // Notify users
    notify
        .send(ChatMessage::new(lobby_id, String::from("User"), msg))
        .await
        .unwrap_or_else(|err| warn!("Can't notify users: {}", err));

    HttpResponse::Ok().json(Success)
}

#[get("/")]
pub async fn test() -> impl Responder {
    debug!("Test ping");
    HttpResponse::Ok().body("Hello, world!")
}
