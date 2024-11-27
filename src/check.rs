use crate::{
    config::ServerConfig,
    notification::{
        internal_messages::{ImageDeleted, SystemNotification, SystemNotificationType},
        server::NotifyServer,
    },
    utils::{delete_img_files, img_id_to_filename},
    ImgId, LobbyId, RoomId, SessionId,
};
use actix::prelude::*;
use actix_web::web::Data;
use image::{DynamicImage, ImageFormat};
use log::{debug, warn};
use reqwest::multipart::Part;
use std::io::Cursor;

pub struct ImgCheck {
    img: DynamicImage,
    lobby_id: LobbyId,
    room_id: RoomId,
    img_id: ImgId,
    uploader_id: Option<SessionId>,
}

impl ImgCheck {
    pub fn new(
        img: DynamicImage,
        lobby_id: LobbyId,
        room_id: RoomId,
        img_id: ImgId,
        uploader_id: Option<SessionId>,
    ) -> Self {
        Self {
            img,
            lobby_id,
            room_id,
            img_id,
            uploader_id,
        }
    }
}

impl Message for ImgCheck {
    type Result = ();
}

#[derive(Debug, Clone)]
pub struct ImgChecker {
    notify: Data<Addr<NotifyServer>>,
    cfg: Data<ServerConfig>,
}

impl ImgChecker {
    pub fn new(notify: Data<Addr<NotifyServer>>, cfg: Data<ServerConfig>) -> Self {
        Self { notify, cfg }
    }
}

impl Actor for ImgChecker {
    type Context = Context<Self>;
}

impl Handler<ImgCheck> for ImgChecker {
    type Result = ();

    fn handle(&mut self, msg: ImgCheck, _ctx: &mut Self::Context) -> Self::Result {
        let notify = self.notify.clone();
        let cfg = self.cfg.clone();
        tokio::spawn(async move {
            let Some(check) = &cfg.upload_check else {
                return;
            };
            // Sende einen GET Request
            let res = check_image(&check.url, msg.img, Some(msg.img_id)).await;

            match res {
                Ok(is_allowed) if !is_allowed => {
                    debug!("Img {} not allowed", msg.img_id);
                    delete_img_files(
                        (msg.lobby_id, msg.room_id, msg.img_id),
                        &cfg.images_storage_path,
                    );
                    notify
                        .send(ImageDeleted::new(msg.lobby_id, msg.room_id, msg.img_id))
                        .await
                        .unwrap_or_else(|err| warn!("Can't notify users: {}", err));
                    if let (Some(fail_msg), Some(uploader_id)) =
                        (check.not_allowed_msg.clone(), msg.uploader_id)
                    {
                        notify
                            .send(SystemNotification::new(
                                uploader_id,
                                fail_msg,
                                SystemNotificationType::Warning,
                            ))
                            .await
                            .unwrap_or_else(|err| warn!("Can't notify uploader: {}", err));
                    }
                }
                Ok(_) => debug!("Img {} allowed", msg.img_id),
                Err(err) => debug!("{err}"),
            };
        });
    }
}

const MIME_STR: &str = "image/jpeg";

pub async fn check_image(
    url: &str,
    img: DynamicImage,
    img_id: Option<ImgId>,
) -> Result<bool, String> {
    let mut buf = Vec::new();
    img.to_rgb8()
        .write_to(&mut Cursor::new(&mut buf), ImageFormat::Jpeg)
        .map_err(|err| format!("{err:?}"))?;
    let content_len = buf.len();
    let img_id = img_id.unwrap_or(0);
    let part = Part::bytes(buf)
        .file_name(img_id_to_filename(img_id))
        .mime_str(MIME_STR)
        .map_err(|err| format!("{err:?}"))?;
    let form = reqwest::multipart::Form::new()
        .text("img_id", img_id.to_string())
        .part("image", part);
    let res = reqwest::Client::new()
        .post(url)
        .header("CONTENT_TYPE", "Multipart/form-data")
        .header("CONTENT_LENGTH", content_len)
        .multipart(form)
        .send()
        .await
        .map_err(|err| format!("{err:?}"))?;
    Ok(res.json::<bool>().await.map_err(|err| format!("{err:?}"))?)
}
