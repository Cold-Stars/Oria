use image::GenericImageView;
use std::path::Path;

use crate::error::AppError;
use crate::models::{AnnotationData, AnnotationFile};
use crate::utils::get_annotation_file_path;

/// 保存标注数据
#[tauri::command]
pub async fn save_annotations(
    image_path: String,
    annotations: Vec<AnnotationData>,
) -> Result<(), AppError> {
    // println!("Saving annotations for: {}", image_path);
    // println!("Annotations count: {}", annotations.len());

    let image_path_obj = Path::new(&image_path);
    if !image_path_obj.exists() {
        return Err(AppError {
            message: format!("Image file does not exist: {}", image_path),
        });
    }

    let image_bytes = std::fs::read(&image_path)?;
    let img = image::load_from_memory(&image_bytes)?;
    let (width, height) = img.dimensions();

    let now = chrono::Utc::now().to_rfc3339();
    let annotation_file = AnnotationFile {
        version: "1.0".to_string(),
        image_path: image_path.clone(),
        image_width: width,
        image_height: height,
        annotations,
        created: now.clone(),
        modified: now,
    };

    let json_data = serde_json::to_string_pretty(&annotation_file)?;
    let annotation_file_path = get_annotation_file_path(&image_path);
    std::fs::write(&annotation_file_path, json_data)?;

    // println!("Annotations saved to: {}", annotation_file_path);
    Ok(())
}

/// 加载标注数据
#[tauri::command]
pub async fn load_annotations(image_path: String) -> Result<Vec<AnnotationData>, AppError> {
    // println!("Loading annotations for: {}", image_path);
    load_annotations_internal(&image_path)
}

/// 内部辅助函数：加载标注数据
pub fn load_annotations_internal(image_path: &str) -> Result<Vec<AnnotationData>, AppError> {
    let annotation_file_path = get_annotation_file_path(image_path);

    if !Path::new(&annotation_file_path).exists() {
        // println!("No annotation file found: {}", annotation_file_path);
        return Ok(Vec::new());
    }

    let file_content = std::fs::read_to_string(&annotation_file_path)?;

    // 尝试加载为AnnotationFile格式（包含元数据）
    let annotation_data: Vec<AnnotationData> =
        match serde_json::from_str::<AnnotationFile>(&file_content) {
            Ok(annotation_file) => {
                // println!(
                //     "Loaded AnnotationFile format with {} annotations",
                //     annotation_file.annotations.len()
                // );
                annotation_file.annotations
            }
            Err(_) => {
                // 如果失败，尝试直接加载为数组格式（旧格式兼容）
                match serde_json::from_str::<Vec<AnnotationData>>(&file_content) {
                    Ok(annotations) => {
                        // println!("Loaded array format with {} annotations", annotations.len());
                        annotations
                    }
                    Err(e) => {
                        return Err(AppError {
                            message: format!("Failed to parse annotation file: {}", e),
                        });
                    }
                }
            }
        };

    // println!("Successfully loaded {} annotations", annotation_data.len());

    Ok(annotation_data)
}

/// 内部辅助函数：保存标注数据
pub fn save_annotations_internal(
    image_path: &str,
    annotations: Vec<AnnotationData>,
) -> Result<(), AppError> {
    // 读取图片尺寸
    let image_path_obj = Path::new(image_path);
    let (width, height) = if image_path_obj.exists() {
        match image::open(image_path_obj) {
            Ok(img) => {
                let (w, h) = img.dimensions();
                (w, h)
            }
            Err(_) => (0, 0), // 如果无法读取图片，使用0作为占位
        }
    } else {
        (0, 0)
    };

    let now = chrono::Utc::now().to_rfc3339();
    let annotation_file = AnnotationFile {
        version: "1.0".to_string(),
        image_path: image_path.to_string(),
        image_width: width,
        image_height: height,
        annotations,
        created: now.clone(),
        modified: now,
    };

    let annotation_file_path = get_annotation_file_path(image_path);
    let json_content = serde_json::to_string_pretty(&annotation_file)?;
    std::fs::write(&annotation_file_path, json_content)?;
    Ok(())
}
