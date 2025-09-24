use crate::{public_messages::permission::ConfirmationResponse, utils::ParamTuple};
use Restriction::*;
use actix_web::{HttpRequest, HttpResponse};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Deserialize, Clone, Default, Debug)]
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

#[derive(Deserialize, Clone, Debug)]
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
                confirm_req.is_allowed(conf_params).await
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

#[derive(Deserialize, Clone, Debug)]
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

#[derive(Deserialize, Clone, Debug)]
pub enum ConfirmationMethod {
    Get,
    Post,
}

#[derive(Deserialize, Clone, Debug)]
pub enum ConfirmationFormat {
    Json,
    Form,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ConfirmationRequest {
    url: String,
    method: ConfirmationMethod,
    format: ConfirmationFormat,
    params: HashMap<String, Value>,
    headers: HashMap<String, String>,
}

impl ConfirmationRequest {
    pub async fn is_allowed(&self, params: HashMap<String, Value>) -> Result<(), String> {
        let client = reqwest::Client::new();

        let mut req = match self.method {
            ConfirmationMethod::Get => client.get(&self.url),
            ConfirmationMethod::Post => client.post(&self.url),
        }
        .headers(self.header_map()?);

        if !params.is_empty() {
            req = match self.format {
                ConfirmationFormat::Json => req.json(&params),
                ConfirmationFormat::Form => req.form(&params),
            };
        }

        let response = &req
            .headers(self.header_map()?)
            .send()
            .await
            .map_err(|err| format!("Can't send confirmation request: {:?}", err))?
            .text()
            .await
            .map_err(|err| format!("Can't read confirmation response: {:?}", err))?;
        let response: ConfirmationResponse = serde_json::from_str(&response).map_err(|err| {
            format!(
                "Can't parse confirmation response: {} | {}",
                err.to_string(),
                response
            )
        })?;

        match response.is_allowed {
            true => Ok(()),
            false => Err(format!(
                "Not allowed: {}",
                response.error_msg.unwrap_or_default()
            )),
        }
    }

    fn header_map(&self) -> Result<HeaderMap, String> {
        let mut headers = HeaderMap::new();
        for (key, value) in self.headers.clone() {
            let header_name =
                HeaderName::from_bytes(key.as_bytes()).map_err(|err| err.to_string())?;
            let header_value = HeaderValue::from_str(&value).map_err(|err| err.to_string())?;
            headers.insert(header_name, header_value);
        }
        return Ok(headers);
    }
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
