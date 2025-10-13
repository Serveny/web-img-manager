use crate::permission::Permissions;
use actix_cors::Cors;
use actix_web::http::header;
use figment::Figment;
use figment::providers::{Env, Format, Json};
use log::error;
use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
#[serde(default)]
pub struct ServerConfig {
    // server url
    pub url: String,

    // server port
    pub port: u16,

    // Path for storing all uploaded images
    pub images_storage_path: String,

    // maximum input image file size
    pub max_image_size_byte: usize,

    // upload permission
    pub permissions: Permissions,

    // Certificate path for openssl
    #[cfg(feature = "openssl")]
    pub cert_pem_path: Option<String>,

    // Private key path for openssl
    #[cfg(feature = "openssl")]
    pub key_pem_path: Option<String>,

    pub upload_check: Option<UploadCheckCfg>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            url: String::from("0.0.0.0"),
            port: 1871,
            images_storage_path: String::from("/wim_storage/pictures"),
            max_image_size_byte: 1024 * 1024 * 20, // 20 MB
            permissions: Permissions::default(),

            #[cfg(feature = "openssl")]
            cert_pem_path: None, // Example: Some(String::from("/wim_storage/cert/cert.pem")),

            #[cfg(feature = "openssl")]
            key_pem_path: None, // Example: Some(String::from("/wim_storage/cert/key.pem")),

            upload_check: None,
        }
    }
}

pub fn read_server_config() -> ServerConfig {
    Figment::new()
        .merge(Json::file("./config/default-server-config.json"))
        .merge(Json::file("./config/server-config.json"))
        .merge(Env::prefixed("WIM_").split("__")) // highest prio
        .extract()
        .unwrap_or_else(|err| {
            error!("{err}. Using hardcoded default config instead");
            ServerConfig::default()
        })
}

pub fn cors_cfg() -> Cors {
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
        .max_age(3600)
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum CheckPhase {
    BeforeUpload,
    AfterUpload,
}

#[derive(Deserialize, Clone, Debug)]
pub struct UploadCheckCfg {
    pub check_phase: CheckPhase,
    pub url: String,
    pub not_allowed_msg: Option<String>,
}
