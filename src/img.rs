use crate::{ImgId, LobbyId, RoomId};
use actix_multipart::form::tempfile::TempFile;
use actix_web::{http::header, HttpResponse};
use image::{imageops::FilterType, DynamicImage, GenericImageView, ImageFormat};
use image_hasher::{HashAlg, HasherConfig, ImageHash};
use log::info;
use std::{
    cmp::Reverse,
    fs::{self, create_dir_all, DirEntry, File},
    io::{BufReader, Error, Read},
    path::{Path, PathBuf},
    time::SystemTime,
};

const IMG_EXTENSION: &str = "webp";

#[derive(PartialEq, Eq)]
pub enum ImgType {
    Big,
    Thumb,
}

pub fn get_img(
    img_type: ImgType,
    params: &(LobbyId, RoomId, ImgId),
    img_storage_path: &str,
) -> HttpResponse {
    let lobby_id = params.0.to_string();
    let room_id = params.1.to_string();
    let img_id = params.2;

    let mut file_base_path = Path::new(img_storage_path).join(lobby_id).join(room_id);
    if img_type == ImgType::Thumb {
        file_base_path = file_base_path.join("thumb");
    }
    file_base_path = file_base_path.join(img_id.to_string());

    // Open file
    let Ok((mut file, filepath)) = open_img(&file_base_path) else {
        return HttpResponse::NotFound().body("Picture not found");
    };

    // Read file content
    let mut img_content = Vec::new();
    let Ok(_) = file.read_to_end(&mut img_content) else {
        return HttpResponse::NoContent().body("Picture file corrupt");
    };

    // Send file back
    HttpResponse::Ok()
        .append_header(header::ContentDisposition::attachment(
            filepath.to_string_lossy().to_string(),
        ))
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

pub enum SaveImageResult {
    Ok(ImgId),
    ImageAlreadyExists(ImgId),
    Err(String),
}

pub fn save_img(
    img: &DynamicImage,
    thumb_img: &DynamicImage,
    lobby_id: &LobbyId,
    room_id: &RoomId,
    img_storage_path: &str,
) -> SaveImageResult {
    // Check storage path
    let storage_path = Path::new(img_storage_path);
    if !storage_path.exists() {
        return SaveImageResult::Err(format!("Storage not found: {img_storage_path}"));
    }

    // Check image folder
    let img_folder_path = storage_path
        .join(lobby_id.to_string())
        .join(room_id.to_string());
    if !img_folder_path.exists() && create_dir_all(&img_folder_path).is_err() {
        return SaveImageResult::Err(String::from("Could not create image folder"));
    }

    // Save big image
    let img_id: ImgId = hash_to_u32(
        HasherConfig::new()
            .hash_alg(HashAlg::Blockhash)
            .hash_size(8, 4)
            .to_hasher()
            .hash_image(img),
    );

    let img_path = img_folder_path.join(img_id_to_filename(img_id));
    if img_path.exists() {
        info!("img_id {img_id} already exists, skip picture");
        return SaveImageResult::ImageAlreadyExists(img_id);
    }
    if let Err(err) = save_as_webp(img, &img_path) {
        return SaveImageResult::Err(err);
    }

    // Check thumb folder
    let thumb_folder_path = img_folder_path.join("thumb");
    if !thumb_folder_path.exists() && create_dir_all(&thumb_folder_path).is_err() {
        return SaveImageResult::Err(String::from("Could not create thumb folder"));
    }

    // Save thumb image
    let thumb_img_path = thumb_folder_path.join(img_id_to_filename(img_id));
    if let Err(err) = save_as_webp(thumb_img, &thumb_img_path) {
        return SaveImageResult::Err(err);
    }

    SaveImageResult::Ok(img_id)
}

fn save_as_webp(img: &DynamicImage, path: &PathBuf) -> Result<(), String> {
    // Create the WebP encoder for the image
    let encoder = webp::Encoder::from_image(img).map_err(|err| err.to_string())?;

    // Encode the image at a specified quality 0-100
    let webp: webp::WebPMemory = encoder.encode(50.);

    std::fs::write(path, &*webp).map_err(|err| err.to_string())
}

fn entry_to_img_id(entry: Result<DirEntry, Error>) -> Option<(ImgId, SystemTime)> {
    let Ok(entry) = entry else {
        return None;
    };

    let img_id = entry
        .file_name()
        .to_string_lossy()
        .to_lowercase()
        .trim_matches(|c: char| !c.is_numeric())
        .parse::<ImgId>();

    let img_time = entry
        .metadata()
        .and_then(|metadata| metadata.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH);

    match img_id {
        Ok(img_id) => Some((img_id, img_time)),
        Err(_) => None,
    }
}

pub fn get_filenames_as_img_id(folder_path: &PathBuf) -> std::io::Result<Vec<ImgId>> {
    let mut entries: Vec<(ImgId, SystemTime)> = fs::read_dir(folder_path)?
        .filter_map(entry_to_img_id)
        .collect();
    entries.sort_by_key(|(_, time)| Reverse(*time));
    Ok(entries.into_iter().map(|(id, _)| id).collect())
}

pub fn delete_img_files(params: (LobbyId, RoomId, ImgId), images_storage_path: &str) {
    let room_path = Path::new(images_storage_path)
        .join(params.0.to_string())
        .join(params.1.to_string());
    let filename = img_id_to_filename(params.2);

    // Delete big image
    let img_path = room_path.join(&filename);
    fs::remove_file(img_path).unwrap_or_default();

    // Delete thumb image
    let thumb_path = room_path.join("thumb").join(filename);
    fs::remove_file(thumb_path).unwrap_or_default();
}

fn hash_to_u32(hash: ImageHash) -> u32 {
    // Take first 32 bit of hash
    let hash_bits = hash.as_bytes();
    let mut hash_u32 = 0u32;

    // Convert to u32
    for (i, &byte) in hash_bits.iter().take(4).enumerate() {
        hash_u32 |= (byte as u32) << (8 * i);
    }

    hash_u32
}

pub fn img_id_to_filename(img_id: ImgId) -> String {
    format!("{}.{}", img_id, IMG_EXTENSION)
}

fn open_img<P: AsRef<Path>>(base_name: P) -> std::io::Result<(File, PathBuf)> {
    // Fallback for jpg files
    let extensions = ["webp", "jpg"];

    for ext in &extensions {
        let file_path = base_name.as_ref().with_extension(ext);
        if file_path.exists() {
            return match File::open(&file_path) {
                Ok(file) => Ok((file, file_path)),
                Err(err) => Err(err),
            };
        }
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "File not found",
    ))
}
