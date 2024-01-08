use crate::config::IMG_STORAGE_PATH;
use base64::{engine::general_purpose::STANDARD as Base64, Engine};
use image::{imageops::FilterType, DynamicImage, GenericImageView, ImageFormat};
use regex::Regex;
use std::{
    fs::{self, create_dir_all},
    path::{Path, PathBuf},
};

pub fn base64_to_img<'a>(base64_img: &'a str) -> Result<DynamicImage, &'static str> {
    // Format
    let re = Regex::new(r#"^data:(.*?);base64,"#).map_err(|_| "Wrong Regex")?;
    let format = re
        .captures(base64_img)
        .and_then(|captures| captures.get(1))
        .map(|match_group| match_group.as_str().trim())
        .ok_or("Wrong format")?;
    println!("{}", format);
    let format = ImageFormat::from_mime_type(format).ok_or("Unknown format")?;

    // Content
    let mut img = base64_img.to_string();
    let offset = base64_img.find(',').unwrap_or(img.len()) + 1;
    img.drain(..offset);
    let img = Base64.decode(img).map_err(|_| "Can't decode image")?;
    let img = image::load_from_memory_with_format(&img, format).map_err(|_| "Can't load image")?;

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
    room_id: &str,
    chapter_id: &str,
) -> Result<u32, String> {
    // Check storage path
    let storage_path = Path::new(IMG_STORAGE_PATH);
    if !storage_path.exists() {
        return Err(String::from("Storage not found"));
    }

    // Check image folder
    let img_folder_path = storage_path.join(room_id).join(chapter_id);
    if !img_folder_path.exists() && create_dir_all(&img_folder_path).is_err() {
        return Err(String::from("Could not create image folder"));
    }

    // Save images
    let img_id = get_filenames(&img_folder_path)
        .iter()
        .map(|name| {
            (*name)
                .to_lowercase()
                .trim_matches(|c: char| !c.is_numeric())
                .parse::<u32>()
                .unwrap_or(0)
        })
        .max()
        .unwrap_or(0)
        + 1;

    let img_path = img_folder_path.join(format!("{}.jpg", img_id));
    if let Err(err) = img.save_with_format(img_path, ImageFormat::Jpeg) {
        return Err(err.to_string());
    }

    let thumb_img_path = img_folder_path.join(format!("{}_thumb.jpg", img_id));
    if let Err(err) = thumb_img.save_with_format(thumb_img_path, ImageFormat::Jpeg) {
        return Err(err.to_string());
    }

    Ok(img_id)
}

pub fn get_filenames(folder_path: &PathBuf) -> Vec<String> {
    fs::read_dir(folder_path)
        .ok()
        .map(|entries| {
            entries
                .filter_map(|entry| {
                    entry
                        .ok()
                        .map(|e| e.file_name().to_string_lossy().into_owned())
                })
                .collect()
        })
        .unwrap_or_else(Vec::new)
}
