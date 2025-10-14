use image::GenericImageView;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::annotation::load_annotations_internal;
use crate::error::AppError;
use crate::models::AnnotationData;
use crate::utils::is_image_file;

/// YOLO导出数据结构
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct YoloExportConfig {
    pub class_map: HashMap<String, u32>, // 类别名称到ID的映射
    pub export_path: String,             // 导出目录路径
}

/// 导出为YOLO格式
#[tauri::command]
pub async fn export_yolo(
    image_path: String,
    annotations: Vec<AnnotationData>,
    config: YoloExportConfig,
) -> Result<String, AppError> {
    // println!("Exporting YOLO format for: {}", image_path);

    let image_path_obj = Path::new(&image_path);
    if !image_path_obj.exists() {
        return Err(AppError {
            message: format!("Image file does not exist: {}", image_path),
        });
    }

    let image_bytes = std::fs::read(&image_path)?;
    let img = image::load_from_memory(&image_bytes)?;
    let (img_width, img_height) = img.dimensions();

    let mut yolo_lines = Vec::new();

    for annotation in &annotations {
        let class_id = config.class_map.get(&annotation.label).unwrap_or(&0);

        let center_x = (annotation.x + annotation.width / 2.0) / img_width as f64;
        let center_y = (annotation.y + annotation.height / 2.0) / img_height as f64;
        let norm_width = annotation.width / img_width as f64;
        let norm_height = annotation.height / img_height as f64;

        let yolo_line = match annotation.annotation_type.as_str() {
            "rectangle" => {
                format!(
                    "{} {:.6} {:.6} {:.6} {:.6}",
                    class_id, center_x, center_y, norm_width, norm_height
                )
            }
            "rotated-rectangle" => {
                // YOLOv8 OBB格式：使用四个角点坐标
                let rotation = annotation.rotation.unwrap_or(0.0);
                let cx = annotation.x + annotation.width / 2.0;
                let cy = annotation.y + annotation.height / 2.0;
                let hw = annotation.width / 2.0;
                let hh = annotation.height / 2.0;

                let cos_r = rotation.cos();
                let sin_r = rotation.sin();

                // 左上、右上、右下、左下
                let x1 = (cx + (-hw * cos_r - (-hh) * sin_r)) / img_width as f64;
                let y1 = (cy + (-hw * sin_r + (-hh) * cos_r)) / img_height as f64;
                let x2 = (cx + (hw * cos_r - (-hh) * sin_r)) / img_width as f64;
                let y2 = (cy + (hw * sin_r + (-hh) * cos_r)) / img_height as f64;
                let x3 = (cx + (hw * cos_r - hh * sin_r)) / img_width as f64;
                let y3 = (cy + (hw * sin_r + hh * cos_r)) / img_height as f64;
                let x4 = (cx + (-hw * cos_r - hh * sin_r)) / img_width as f64;
                let y4 = (cy + (-hw * sin_r + hh * cos_r)) / img_height as f64;

                format!(
                    "{} {:.6} {:.6} {:.6} {:.6} {:.6} {:.6} {:.6} {:.6}",
                    class_id, x1, y1, x2, y2, x3, y3, x4, y4
                )
            }
            _ => continue,
        };

        yolo_lines.push(yolo_line);
    }

    let image_filename = image_path_obj.file_stem().unwrap_or_default();
    let output_path =
        Path::new(&config.export_path).join(format!("{}.txt", image_filename.to_string_lossy()));

    let yolo_content = yolo_lines.join("\n");
    std::fs::write(&output_path, yolo_content)?;

    let output_path_str = output_path.to_string_lossy().to_string();
    // println!("YOLO file exported to: {}", output_path_str);

    Ok(output_path_str)
}

/// 批量导出YOLO格式
#[tauri::command]
pub async fn export_yolo_batch(
    image_folder: String,
    config: YoloExportConfig,
) -> Result<Vec<String>, AppError> {
    // println!("Batch exporting YOLO format from folder: {}", image_folder);

    let folder_path = Path::new(&image_folder);
    if !folder_path.exists() {
        return Err(AppError {
            message: format!("Folder does not exist: {}", image_folder),
        });
    }

    std::fs::create_dir_all(&config.export_path)?;

    let mut exported_files = Vec::new();
    let entries = std::fs::read_dir(folder_path)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && is_image_file(&path) {
            let image_path_str = path.to_string_lossy().to_string();

            match load_annotations_internal(&image_path_str) {
                Ok(annotations) => {
                    if !annotations.is_empty() {
                        match export_yolo(image_path_str, annotations, config.clone()).await {
                            Ok(output_path) => {
                                exported_files.push(output_path);
                            }
                            Err(e) => {
                                println!(
                                    "Failed to export YOLO for {}: {}",
                                    path.display(),
                                    e.message
                                );
                            }
                        }
                    }
                }
                Err(e) => {
                    println!(
                        "Failed to load annotations for {}: {}",
                        path.display(),
                        e.message
                    );
                }
            }
        }
    }

    // println!(
    //     "Batch export completed. {} files exported.",
    //     exported_files.len()
    // );
    Ok(exported_files)
}
