use crate::{ImgId, LobbyId, RoomId, SessionId};
use actix_web::HttpRequest;
use serde_json::{from_value, Value};
use std::{
    collections::HashMap,
    fs::{self, DirEntry},
    io::Error,
    path::PathBuf,
};

pub trait ToOutputJsonString {
    fn to_output_json_string(&self) -> Result<String, serde_json::Error>;
}

pub fn get_foldernames_as_uuid(folder_path: &PathBuf) -> Vec<RoomId> {
    let entry_to_room_id = |entry: Result<DirEntry, Error>| {
        entry.ok().and_then(|e| match e.path().is_dir() {
            true => e.file_name().to_string_lossy().parse::<RoomId>().ok(),
            false => None,
        })
    };
    fs::read_dir(folder_path)
        .ok()
        .map(|entries| entries.filter_map(entry_to_room_id).collect())
        .unwrap_or_else(Vec::new)
}

pub fn rename_with_value<T: Into<Value>>(map: &mut HashMap<String, Value>, key: &str, val: T) {
    if let Some(new_key) = map.get_mut(key) {
        if let Ok(new_key) = from_value::<String>(new_key.clone()) {
            map.insert(new_key, val.into());
            map.remove(key);
        } else {
            *new_key = val.into();
        }
    }
}

pub trait ParamTuple {
    fn edit_param_map(&self, map: &mut HashMap<String, Value>);
}

impl ParamTuple for (LobbyId,) {
    fn edit_param_map(&self, map: &mut HashMap<String, Value>) {
        rename_with_value(map, "lobby_id", self.0.to_string());
    }
}

impl ParamTuple for (LobbyId, RoomId) {
    fn edit_param_map(&self, map: &mut HashMap<String, Value>) {
        rename_with_value(map, "lobby_id", self.0.to_string());
        rename_with_value(map, "room_id", self.1);
    }
}

impl ParamTuple for (LobbyId, RoomId, ImgId) {
    fn edit_param_map(&self, map: &mut HashMap<String, Value>) {
        rename_with_value(map, "lobby_id", self.0.to_string());
        rename_with_value(map, "room_id", self.1);
        rename_with_value(map, "img_id", self.2);
    }
}

pub const SESSION_COOKIE_NAME: &str = "wim_session_id";

pub fn get_session_id(req: &HttpRequest) -> Option<SessionId> {
    req.cookie(SESSION_COOKIE_NAME)
        .and_then(|cookie| SessionId::parse_str(cookie.value()).ok())
}
