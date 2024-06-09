use crate::{config::IMG_STORAGE_PATH, ImgId, LobbyId, RoomId};
use actix_multipart::form::tempfile::TempFile;
use actix_web::{
    http::header,
    web::{self},
    HttpResponse, Responder,
};
use image::{imageops::FilterType, DynamicImage, GenericImageView, ImageFormat};
use std::{
    fs::{self, create_dir_all, DirEntry, File},
    io::{BufReader, Error, Read},
    path::{Path, PathBuf},
};
use uuid::Uuid;

#[derive(PartialEq, Eq)]
pub enum ImgType {
    Big,
    Thumb,
}
pub trait ToOutputJsonString {
    fn to_output_json_string(&self) -> Result<String, serde_json::Error>;
}

pub fn get_img(img_type: ImgType, info: web::Path<(LobbyId, RoomId, ImgId)>) -> impl Responder {
    let lobby_id = info.0.to_string();
    let room_id = info.1.to_string();
    let filename = format!("{}.jpg", info.2);

    let mut file_path = Path::new(IMG_STORAGE_PATH).join(lobby_id).join(room_id);
    if img_type == ImgType::Thumb {
        file_path = file_path.join("thumb");
    }
    file_path = file_path.join(&filename);

    // Open file
    let Ok(mut file) = File::open(&file_path) else {
        return HttpResponse::NotFound().body("Picture not found");
    };

    // Read file content
    let mut img_content = Vec::new();
    let Ok(_) = file.read_to_end(&mut img_content) else {
        return HttpResponse::NoContent().body("Picture file corrupt");
    };

    // Send file back
    HttpResponse::Ok()
        .append_header(header::ContentType::jpeg())
        .append_header(header::ContentDisposition::attachment(filename))
        .body(img_content)
}

pub fn read_img(temp_file: &TempFile) -> Result<DynamicImage, &'static str> {
    let Ok(file) = std::fs::File::open(&temp_file.file) else {
        return Err("Cannot read file");
    };
    let reader = BufReader::new(file);
    let format = temp_file
        .content_type
        .as_ref()
        .ok_or("Can't read image format")?;
    let format = ImageFormat::from_mime_type(format).ok_or("Unknown image format")?;
    let img = image::load(reader, format).map_err(|_| "Image corrupt")?;

    Ok(img)
}

pub fn resize_image(img: DynamicImage, max_width: u32, max_height: u32) -> DynamicImage {
    let (width, height) = img.dimensions();

    if width > max_width || height > max_height {
        return img.resize(max_width, max_height, FilterType::Triangle);
    }

    img
}

pub fn save_img(
    img: DynamicImage,
    thumb_img: DynamicImage,
    lobby_id: &Uuid,
    room_id: &Uuid,
) -> Result<u32, String> {
    // Check storage path
    let storage_path = Path::new(IMG_STORAGE_PATH);
    if !storage_path.exists() {
        return Err(String::from("Storage not found"));
    }

    // Check image folder
    let img_folder_path = storage_path
        .join(lobby_id.to_string())
        .join(room_id.to_string());
    if !img_folder_path.exists() && create_dir_all(&img_folder_path).is_err() {
        return Err(String::from("Could not create image folder"));
    }

    // Save big image
    let img_id = get_filenames_as_u32(&img_folder_path)
        .iter()
        .max()
        .unwrap_or(&0)
        + 1;

    let img_path = img_folder_path.join(img_id_to_filename(img_id));
    if let Err(err) = img.to_rgb8().save_with_format(img_path, ImageFormat::Jpeg) {
        return Err(err.to_string());
    }

    // Check thumb folder
    let thumb_folder_path = img_folder_path.join("thumb");
    if !thumb_folder_path.exists() && create_dir_all(&thumb_folder_path).is_err() {
        return Err(String::from("Could not create thumb folder"));
    }

    // Save thumb image
    let thumb_img_path = thumb_folder_path.join(format!("{}.jpg", img_id));
    if let Err(err) = thumb_img
        .to_rgb8()
        .save_with_format(thumb_img_path, ImageFormat::Jpeg)
    {
        return Err(err.to_string());
    }

    Ok(img_id)
}

pub fn get_filenames_as_u32(folder_path: &PathBuf) -> Vec<ImgId> {
    let entry_to_u32 = |entry: Result<DirEntry, Error>| {
        entry.ok().and_then(|e| {
            e.file_name()
                .to_string_lossy()
                .to_lowercase()
                .trim_matches(|c: char| !c.is_numeric())
                .parse::<u32>()
                .ok()
        })
    };
    fs::read_dir(folder_path)
        .ok()
        .map(|entries| entries.filter_map(entry_to_u32).collect())
        .unwrap_or_else(Vec::new)
}

pub fn get_foldernames_as_uuid(folder_path: &PathBuf) -> Vec<RoomId> {
    let entry_to_uuid = |entry: Result<DirEntry, Error>| {
        entry.ok().and_then(|e| match e.path().is_dir() {
            true => e.file_name().to_string_lossy().parse::<Uuid>().ok(),
            false => None,
        })
    };
    fs::read_dir(folder_path)
        .ok()
        .map(|entries| entries.filter_map(entry_to_uuid).collect())
        .unwrap_or_else(Vec::new)
}

pub fn img_id_to_filename(img_id: ImgId) -> String {
    format!("{}.jpg", img_id)
}
