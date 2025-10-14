use std::path::Path;

/// 支持的图片格式
pub const SUPPORTED_IMAGE_EXTENSIONS: &[&str] =
    &["jpg", "jpeg", "png", "bmp", "gif", "tiff", "tif", "webp"];

/// 检查文件是否为支持的图片格式
pub fn is_image_file(path: &Path) -> bool {
    if let Some(extension) = path.extension() {
        if let Some(ext_str) = extension.to_str() {
            return SUPPORTED_IMAGE_EXTENSIONS.contains(&ext_str.to_lowercase().as_str());
        }
    }
    false
}

/// 获取标注文件路径
pub fn get_annotation_file_path(image_path: &str) -> String {
    let image_path = Path::new(image_path);
    let parent_dir = image_path.parent().unwrap_or_else(|| Path::new("."));
    let file_stem = image_path.file_stem().unwrap_or_default();

    parent_dir
        .join(format!("{}.json", file_stem.to_string_lossy()))
        .to_string_lossy()
        .to_string()
}
