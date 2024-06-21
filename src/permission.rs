use crate::{
    public_messages::permission::ConfirmationResponse, utils::ParamTuple, LobbyId, RoomId,
};
use actix_web::{HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use Restriction::*;

#[derive(Deserialize, Clone, Default)]
pub struct Permissions {
    pub get_room_list: Permission,
    pub get_room_img_list: Permission,
    pub get_img_thumb: Permission,
    pub get_img_big: Permission,
    pub upload_img: Permission,
    pub delete_lobby: Permission,
    pub delete_room: Permission,
    pub delete_img: Permission,
    pub send_chat_message: Permission,
}

#[derive(Deserialize, Clone)]
pub struct Permission {
    restriction: Restriction,
    url_whitelist: Option<Vec<String>>,
}

impl Permission {
    pub async fn is_allowed<T: ParamTuple>(
        &self,
        req: &HttpRequest,
        params: &T,
    ) -> Result<(), String> {
        self.is_allowed_url(req).and(match &self.restriction {
            AllowedToAll => return Ok(()),
            NeedsConfirmation(confirm_req) => {
                let mut conf_params = confirm_req.params.clone();
                params.edit_param_map(&mut conf_params);
                confirm_req.is_allowed(&conf_params).await
            }
            Denied => Err(self.restriction.to_string()),
        })
    }

    fn is_allowed_url(&self, req: &HttpRequest) -> Result<(), String> {
        let Some(urls) = &self.url_whitelist else {
            return Ok(());
        };

        let referer = req
            .headers()
            .get("Referer")
            .ok_or("No referer".to_string())?;

        let referer = referer.to_str().map_err(|err| err.to_string())?;

        urls.contains(&referer.to_string())
            .then(|| ())
            .ok_or("Access from this url not allowed".to_string())
    }
}

impl Default for Permission {
    fn default() -> Self {
        Self {
            restriction: Restriction::AllowedToAll,
            url_whitelist: None,
        }
    }
}

#[derive(Deserialize, Clone)]
pub enum Restriction {
    AllowedToAll,
    NeedsConfirmation(ConfirmationRequest),
    Denied,
}

impl Restriction {
    fn to_string(&self) -> String {
        String::from(match self {
            AllowedToAll => "Allowed",
            NeedsConfirmation(_) => "Needs confirmation from other server",
            Denied => "Access denied",
        })
    }
}

#[derive(Deserialize, Clone)]
pub struct ConfirmationRequest {
    url: String,
    params: HashMap<String, Value>,
}

impl ConfirmationRequest {
    async fn is_allowed(&self, params: &HashMap<String, Value>) -> Result<(), String> {
        let client = reqwest::Client::new();
        let req = client.post(&self.url).json(params);

        let response = &req
            .send()
            .await
            .map_err(|err| err.to_string())?
            .text()
            .await
            .map_err(|err| err.to_string())?;
        let response: ConfirmationResponse = serde_json::from_str(&response)
            .map_err(|err| format!("Can't parse response: {} | {}", err.to_string(), response))?;

        match response.is_allowed {
            true => Ok(()),
            false => Err(format!(
                "Not allowed: {}",
                response.error_msg.unwrap_or_default()
            )),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct UploadConfirmationRequest {
    lobby_id: LobbyId,
    room_id: RoomId,
}

pub async fn check<T: ParamTuple>(
    permission: &Permission,
    req: &HttpRequest,
    params: &T,
) -> Option<HttpResponse> {
    if let Err(msg) = permission.is_allowed(&req, params).await {
        return Some(HttpResponse::Forbidden().body(msg));
    }
    None
}
