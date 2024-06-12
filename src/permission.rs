use crate::{LobbyId, RoomId};
use actix_web::HttpRequest;
use serde::{Deserialize, Serialize};
use serde_json::{from_value, Value};
use std::collections::HashMap;
use Restriction::*;

#[derive(Deserialize, Clone)]
pub struct Permission {
    restriction: Restriction,
    url_whitelist: Option<Vec<String>>,
}

impl Permission {
    pub async fn is_allowed(
        &self,
        req: &HttpRequest,
        lobby_id: LobbyId,
        room_id: RoomId,
    ) -> Result<(), String> {
        self.is_allowed_url(req).and(match &self.restriction {
            AllowedToAll => return Ok(()),
            NeedsConfirmation(confirm_req) => {
                let mut params = confirm_req.params.clone();
                rename_with_value(&mut params, "lobby_id", lobby_id.to_string());
                rename_with_value(&mut params, "room_id", room_id.to_string());
                confirm_req.is_allowed(&params).await
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

        serde_json::from_str(
            &req.send()
                .await
                .map_err(|err| err.to_string())?
                .text()
                .await
                .map_err(|err| err.to_string())?,
        )
        .map_err(|err| err.to_string())
    }
}

#[derive(Serialize, Deserialize)]
pub struct ConfirmationResponse {
    is_allowed: bool,
    msg: String,
}

#[derive(Serialize, Deserialize)]
pub struct UploadConfirmationRequest {
    lobby_id: LobbyId,
    room_id: RoomId,
}

fn rename_with_value(map: &mut HashMap<String, Value>, key: &str, val: String) {
    if let Some(new_key) = map.get_mut(key) {
        if let Ok(new_key) = from_value::<String>(new_key.clone()) {
            map.insert(new_key, Value::String(val));
        } else {
            *new_key = Value::String(val);
        }
    }
}
