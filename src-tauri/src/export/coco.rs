use image::GenericImageView;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::annotation::{load_annotations_internal, save_annotations_internal};
use crate::error::AppError;
use crate::models::AnnotationData;
use crate::utils::is_image_file;

/// COCO格式数据结构
#[derive(Debug, Serialize, Deserialize)]
pub struct CocoFormat {
    pub images: Vec<CocoImage>,
    pub annotations: Vec<CocoAnnotation>,
    pub categories: Vec<CocoCategory>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CocoImage {
    pub id: u32,
    pub file_name: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CocoAnnotation {
    pub id: u32,
    pub image_id: u32,
    pub category_id: u32,
    pub segmentation: Vec<Vec<f64>>, // 用于旋转框的多边形表示，矩形框为空数组
    pub area: f64,
    pub bbox: Vec<f64>, // [x, y, width, height]
    pub iscrowd: u8,
    pub attributes: CocoAttributes, // CVAT等工具导出的额外属性
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CocoAttributes {
    pub occluded: bool,
    pub rotation: f64, // 旋转角度（在attributes中，单位：度）
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CocoCategory {
    pub id: u32,
    pub name: String,
    pub supercategory: String,
}

/// 导出COCO格式
#[tauri::command]
pub async fn export_coco(image_folder: String, export_path: String) -> Result<String, AppError> {
    // println!("Exporting COCO format from folder: {}", image_folder);

    let folder_path = Path::new(&image_folder);
    if !folder_path.exists() {
        return Err(AppError {
            message: format!("Folder does not exist: {}", image_folder),
        });
    }

    let mut coco_images = Vec::new();
    let mut coco_annotations = Vec::new();
    let mut categories_map: HashMap<String, u32> = HashMap::new();
    let mut image_id_counter = 1u32;
    let mut annotation_id_counter = 1u32;
    let mut category_id_counter = 1u32;
    let mut has_rotated = false;

    let entries = std::fs::read_dir(folder_path)?;
    let mut _total_images = 0;
    let mut _processed_images = 0;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && is_image_file(&path) {
            _total_images += 1;
            let image_path_str = path.to_string_lossy().to_string();
            let file_name = path.file_name().unwrap().to_string_lossy().to_string();

            // println!("Processing image: {}", file_name);

            let image_bytes = match std::fs::read(&path) {
                Ok(bytes) => bytes,
                Err(e) => {
                    println!("Failed to read image {}: {}", file_name, e);
                    continue;
                }
            };

            let img = match image::load_from_memory(&image_bytes) {
                Ok(img) => img,
                Err(e) => {
                    println!("Failed to load image {}: {}", file_name, e);
                    continue;
                }
            };

            let (img_width, img_height) = img.dimensions();

            let annotations = match load_annotations_internal(&image_path_str) {
                Ok(anns) => anns,
                Err(e) => {
                    println!(
                        "Failed to load annotations for {}: {}",
                        file_name, e.message
                    );
                    continue;
                }
            };

            if annotations.is_empty() {
                // println!("No annotations found for: {}", file_name);
                continue;
            }

            //             println!("Found {} annotations for: {}", annotations.len(), file_name);
            _processed_images += 1;

            for ann in &annotations {
                if ann.annotation_type == "rotated-rectangle" {
                    has_rotated = true;
                    break;
                }
            }

            coco_images.push(CocoImage {
                id: image_id_counter,
                file_name: file_name.clone(),
                width: img_width,
                height: img_height,
            });

            for annotation in annotations {
                let category_id = if let Some(&id) = categories_map.get(&annotation.label) {
                    id
                } else {
                    let id = category_id_counter;
                    categories_map.insert(annotation.label.clone(), id);
                    category_id_counter += 1;
                    id
                };

                let bbox = vec![
                    annotation.x,
                    annotation.y,
                    annotation.width,
                    annotation.height,
                ];

                let area = annotation.width * annotation.height;

                // 统一使用空的segmentation
                let segmentation = vec![];

                // 计算旋转角度（弧度转为度）
                let rotation_degrees = if annotation.annotation_type == "rotated-rectangle" {
                    let rot = annotation.rotation.unwrap_or(0.0);
                    rot * 180.0 / std::f64::consts::PI
                } else {
                    0.0
                };

                let attributes = CocoAttributes {
                    occluded: false,
                    rotation: rotation_degrees,
                };

                coco_annotations.push(CocoAnnotation {
                    id: annotation_id_counter,
                    image_id: image_id_counter,
                    category_id,
                    segmentation,
                    area,
                    bbox,
                    iscrowd: 0,
                    attributes,
                });

                annotation_id_counter += 1;
            }

            image_id_counter += 1;
        }
    }

    let mut categories: Vec<CocoCategory> = categories_map
        .into_iter()
        .map(|(name, id)| CocoCategory {
            id,
            name: name.clone(),
            supercategory: if has_rotated {
                "rotated-object".to_string()
            } else {
                "object".to_string()
            },
        })
        .collect();

    categories.sort_by_key(|c| c.id);

    let coco_data = CocoFormat {
        images: coco_images,
        annotations: coco_annotations,
        categories,
    };

    // println!("\n=== COCO Export Summary ===");
    // println!("Total images found: {}", total_images);
    // println!("Images with annotations: {}", processed_images);
    // println!("Total annotations: {}", coco_data.annotations.len());
    // println!("Total categories: {}", coco_data.categories.len());
    // println!(
    //     "Categories: {:?}",
    //     coco_data
    //         .categories
    //         .iter()
    //         .map(|c| &c.name)
    //         .collect::<Vec<_>>()
    // );
    // println!("==========================\n");

    let output_file = Path::new(&export_path).join("annotations.json");
    let json_content = serde_json::to_string_pretty(&coco_data)?;
    std::fs::write(&output_file, json_content)?;

    let output_path_str = output_file.to_string_lossy().to_string();
    // println!("COCO format exported to: {}", output_path_str);

    Ok(output_path_str)
}

/// 导入COCO格式
/// import_type: "auto"（自动判断）、"rectangle"（强制矩形框）、"rotated-rectangle"（强制旋转框）
#[tauri::command]
pub async fn import_coco(
    coco_file_path: String,
    image_folder: String,
    import_type: String,
) -> Result<String, AppError> {
    // println!("Importing COCO format from: {}", coco_file_path);

    let coco_content = std::fs::read_to_string(&coco_file_path)?;
    let coco_data: CocoFormat = serde_json::from_str(&coco_content)?;

    let category_map: HashMap<u32, String> = coco_data
        .categories
        .iter()
        .map(|c| (c.id, c.name.clone()))
        .collect();

    let image_map: HashMap<u32, &CocoImage> =
        coco_data.images.iter().map(|img| (img.id, img)).collect();

    let folder_path = Path::new(&image_folder);
    let mut imported_count = 0;

    let mut annotations_by_image: HashMap<u32, Vec<&CocoAnnotation>> = HashMap::new();
    for ann in &coco_data.annotations {
        annotations_by_image
            .entry(ann.image_id)
            .or_insert_with(Vec::new)
            .push(ann);
    }

    let mut total_annotations = 0;
    let mut rotated_count = 0;
    let mut rectangle_count = 0;
    let mut _scaled_images = 0;
    let mut debug_info = Vec::new();

    for (image_id, anns) in annotations_by_image {
        let image_info = match image_map.get(&image_id) {
            Some(info) => info,
            None => continue,
        };

        let image_path = folder_path.join(&image_info.file_name);
        if !image_path.exists() {
            // println!("Image not found: {:?}", image_path);
            continue;
        }

        // 读取实际图片尺寸，用于坐标缩放
        let actual_dimensions = match image::open(&image_path) {
            Ok(img) => {
                let (w, h) = img.dimensions();
                (w as f64, h as f64)
            }
            Err(e) => {
                println!(
                    "Failed to read image dimensions for {:?}: {}",
                    image_path, e
                );
                continue;
            }
        };

        // 计算缩放比例
        let scale_x = actual_dimensions.0 / image_info.width as f64;
        let scale_y = actual_dimensions.1 / image_info.height as f64;

        // 如果缩放比例不是1，说明图片尺寸发生了变化
        let needs_scaling = (scale_x - 1.0).abs() > 0.001 || (scale_y - 1.0).abs() > 0.001;

        // 记录调试信息（前3张图片）
        if debug_info.len() < 3 {
            debug_info.push(format!(
                "{}：COCO尺寸={}x{}, 实际尺寸={}x{}, 缩放={}x{}",
                image_info.file_name,
                image_info.width,
                image_info.height,
                actual_dimensions.0 as u32,
                actual_dimensions.1 as u32,
                if needs_scaling {
                    format!("{:.2}", scale_x)
                } else {
                    "1.00".to_string()
                },
                if needs_scaling {
                    format!("{:.2}", scale_y)
                } else {
                    "1.00".to_string()
                }
            ));
        }

        if needs_scaling {
            _scaled_images += 1;
        }

        let mut converted_annotations = Vec::new();
        for ann in anns {
            total_annotations += 1;
            let category_name = category_map
                .get(&ann.category_id)
                .cloned()
                .unwrap_or_else(|| format!("category_{}", ann.category_id));

            // 获取旋转角度（从attributes.rotation）
            let rotation_degrees = Some(ann.attributes.rotation);

            // 根据用户选择的导入类型决定标注类型
            let annotation_type = match import_type.as_str() {
                "rectangle" => "rectangle",
                "rotated-rectangle" => "rotated-rectangle",
                "auto" | _ => {
                    // 自动判断：如果有旋转角度（非0），则为旋转框
                    let has_rotation =
                        rotation_degrees.is_some() && rotation_degrees.unwrap().abs() > 0.001;
                    if has_rotation {
                        "rotated-rectangle"
                    } else {
                        "rectangle"
                    }
                }
            };

            // 处理旋转角度（转换为弧度）
            let rotation = if annotation_type == "rotated-rectangle" {
                // 如果是旋转框，将角度从度转换为弧度
                if let Some(deg) = rotation_degrees {
                    Some(deg * std::f64::consts::PI / 180.0)
                } else {
                    // 强制为旋转框时，即使原数据没有角度，也设置为0度
                    Some(0.0)
                }
            } else {
                None
            };

            // 统计类型
            if annotation_type == "rotated-rectangle" {
                rotated_count += 1;
            } else {
                rectangle_count += 1;
            }

            // 应用坐标缩放（如果需要）
            let scaled_x = ann.bbox[0] * scale_x;
            let scaled_y = ann.bbox[1] * scale_y;
            let scaled_width = ann.bbox[2] * scale_x;
            let scaled_height = ann.bbox[3] * scale_y;

            // 调试输出（前2个标注）
            if debug_info.len() <= 3 && converted_annotations.len() < 2 {
                debug_info.push(format!(
                    "  标注{}: 原始=[{:.0},{:.0},{:.0},{:.0}], 缩放后=[{:.0},{:.0},{:.0},{:.0}]",
                    ann.id,
                    ann.bbox[0],
                    ann.bbox[1],
                    ann.bbox[2],
                    ann.bbox[3],
                    scaled_x,
                    scaled_y,
                    scaled_width,
                    scaled_height
                ));
            }

            let annotation_data = AnnotationData {
                id: format!("{}", ann.id),
                annotation_type: annotation_type.to_string(),
                x: scaled_x,
                y: scaled_y,
                width: scaled_width,
                height: scaled_height,
                rotation,
                label: category_name,
                created: chrono::Utc::now().to_rfc3339(),
                visible: true,
            };

            converted_annotations.push(annotation_data);
        }

        let image_path_str = image_path.to_string_lossy().to_string();
        match save_annotations_internal(&image_path_str, converted_annotations) {
            Ok(_) => imported_count += 1,
            Err(e) => println!(
                "Failed to save annotations for {:?}: {}",
                image_path, e.message
            ),
        }
    }

    let result_msg = format!(
        "成功导入 {} 张图片，共 {} 个标注\n矩形框: {}, 旋转框: {}",
        imported_count, total_annotations, rectangle_count, rotated_count
    );

    Ok(result_msg)
}
