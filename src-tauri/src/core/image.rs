use base64::{engine::general_purpose, Engine as _};
use image::GenericImageView;
use std::path::Path;

use crate::error::AppError;
use crate::models::ImageData;

/// 加载图片并返回base64数据
#[tauri::command]
pub async fn load_image(path: String) -> Result<ImageData, AppError> {
    // println!("Loading image: {}", path);

    let image_path = Path::new(&path);
    if !image_path.exists() {
        return Err(AppError {
            message: format!("Image file does not exist: {}", path),
        });
    }

    let image_bytes = std::fs::read(&path)?;
    let img = image::load_from_memory(&image_bytes)?;
    let (width, height) = img.dimensions();

    let base64_data = general_purpose::STANDARD.encode(&image_bytes);

    let mime_type = match image_path.extension().and_then(|ext| ext.to_str()) {
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("bmp") => "image/bmp",
        Some("gif") => "image/gif",
        Some("tiff") | Some("tif") => "image/tiff",
        Some("webp") => "image/webp",
        _ => "image/jpeg",
    };

    let data_url = format!("data:{};base64,{}", mime_type, base64_data);

    // println!("Image loaded successfully: {}x{}", width, height);

    Ok(ImageData {
        url: data_url,
        width,
        height,
        file_path: path,
    })
}
