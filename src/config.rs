use crate::permission::Permissions;
use serde::Deserialize;
use std::fs;

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
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            url: String::from("0.0.0.0"),
            port: 1870,
            images_storage_path: String::from("/wim_storage/pictures"),
            max_image_size_byte: 1024 * 1024 * 20, // 20 MB
            permissions: Permissions::default(),
        }
    }
}

pub fn read_server_config() -> Result<ServerConfig, String> {
    let cfg_path = match fs::read_to_string("./config/server-config.json") {
        Ok(cfg) => cfg,
        Err(_) => match fs::read_to_string("./config/default-server-config.json") {
            Ok(default_cfg) => {
                println!("use default-server-config.json");
                default_cfg
            }
            Err(err) => return Err(format!("Can't read config file: {err}")),
        },
    };
    let cfg = match serde_json::from_str(&cfg_path) {
        Ok(cfg) => cfg,
        Err(err) => return Err(format!("Invalid config json: {err}")),
    };
    Ok(cfg)
}
