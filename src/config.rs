use crate::permission::Permissions;
use actix_cors::Cors;
use actix_web::http::header;
use serde::Deserialize;
use std::{env::current_dir, fs};

#[derive(Deserialize, Clone)]
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
            cert_pem_path: Some(String::from("/wim_storage/cert/cert.pem")),

            #[cfg(feature = "openssl")]
            key_pem_path: Some(String::from("/wim_storage/cert/key.pem")),
        }
    }
}

pub fn read_server_config() -> Result<ServerConfig, String> {
    let cfg_json = match fs::read_to_string("./config/server-config.json") {
        Ok(cfg) => cfg,
        Err(_) => match fs::read_to_string("./config/default-server-config.json") {
            Ok(default_cfg) => {
                println!("use default-server-config.json");
                default_cfg
            }
            Err(err) => {
                let dir = current_dir()
                    .map_err(|err| err.to_string())?
                    .join("./config/default-server-config.json");
                let dir = dir.to_str().ok_or(String::from("Can't read config dir"))?;
                return Err(format!("Can't read config file: {err} {dir}",));
            }
        },
    };
    let cfg = match serde_json::from_str(&cfg_json) {
        Ok(cfg) => cfg,
        Err(err) => return Err(format!("Invalid config json: {err}")),
    };
    Ok(cfg)
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
