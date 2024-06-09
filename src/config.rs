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

    // password to use admin commands
    pub admin_pw: String,

    // maximum input image file size
    pub max_image_size_byte: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            url: String::from("127.0.0.1"),
            port: 8080,
            images_storage_path: String::from("./img-storage"),
            admin_pw: String::from("1234"),
            max_image_size_byte: 1024 * 1024 * 20, // 20 MB
        }
    }
}

pub fn read_server_config() -> Result<ServerConfig, &'static str> {
    let Ok(cfg_file) = fs::read_to_string("./config/server-config.json") else {
        return Err("Can't read config file");
    };
    let Ok(cfg) = serde_json::from_str(&cfg_file) else {
        return Err("Invalid config json");
    };
    Ok(cfg)
}
