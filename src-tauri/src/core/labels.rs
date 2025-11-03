use std::fs;
use std::path::{Path, PathBuf};

use crate::error::AppError;

/// 获取 classes.txt 文件路径
pub fn get_classes_file_path(image_folder: &str) -> PathBuf {
    Path::new(image_folder).join("classes.txt")
}

/// 读取 classes.txt 文件
/// 返回标签列表，每行一个标签
#[tauri::command]
pub async fn read_classes_file(image_folder: String) -> Result<Vec<String>, AppError> {
    let classes_path = get_classes_file_path(&image_folder);
    
    if !classes_path.exists() {
        // 如果文件不存在，返回空列表
        return Ok(Vec::new());
    }
    
    let content = fs::read_to_string(&classes_path)?;
    
    // 按行分割，过滤空行和空白行
    let labels: Vec<String> = content
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .map(|line| line.to_string())
        .collect();
    
    Ok(labels)
}

/// 写入 classes.txt 文件
/// 每行一个标签
#[tauri::command]
pub async fn write_classes_file(
    image_folder: String,
    labels: Vec<String>,
) -> Result<(), AppError> {
    let classes_path = get_classes_file_path(&image_folder);
    
    // 过滤空标签并去重
    let mut unique_labels: Vec<String> = labels
        .into_iter()
        .map(|label| label.trim().to_string())
        .filter(|label| !label.is_empty())
        .collect();
    
    // 去重但保持顺序
    unique_labels.sort();
    unique_labels.dedup();
    
    // 写入文件，每行一个标签
    let content = unique_labels.join("\n");
    fs::write(&classes_path, content)?;
    
    Ok(())
}

/// 追加标签到 classes.txt
/// 如果标签已存在，不会重复添加
#[tauri::command]
pub async fn append_class_label(
    image_folder: String,
    label: String,
) -> Result<(), AppError> {
    let label = label.trim();
    if label.is_empty() {
        return Ok(());
    }
    
    // 读取现有标签
    let mut labels = read_classes_file(image_folder.clone()).await?;
    
    // 检查是否已存在
    if !labels.contains(&label.to_string()) {
        labels.push(label.to_string());
        // 写回文件
        write_classes_file(image_folder, labels).await?;
    }
    
    Ok(())
}

/// 从文件夹中的所有 JSON 文件提取标签
/// 用于初始化或重建 classes.txt
#[tauri::command]
pub async fn extract_labels_from_folder(image_folder: String) -> Result<Vec<String>, AppError> {
    use std::collections::HashSet;
    use crate::core::annotation::load_annotations_internal;
    
    let folder_path = Path::new(&image_folder);
    if !folder_path.exists() || !folder_path.is_dir() {
        return Err(AppError {
            message: format!("Invalid folder path: {}", image_folder),
        });
    }
    
    let mut all_labels = HashSet::new();
    
    // 遍历文件夹中的所有文件
    let entries = fs::read_dir(folder_path)?;
    
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        
        // 只处理 JSON 文件
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            // 尝试找到对应的图片文件
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                // 尝试常见的图片扩展名
                let image_extensions = ["jpg", "jpeg", "png", "bmp", "webp", "gif"];
                
                for ext in &image_extensions {
                    let image_path = folder_path.join(format!("{}.{}", stem, ext));
                    if image_path.exists() {
                        // 加载标注数据
                        if let Ok(annotations) = load_annotations_internal(
                            image_path.to_str().unwrap_or("")
                        ) {
                            // 提取标签
                            for annotation in annotations {
                                let label = annotation.label.trim();
                                if !label.is_empty() {
                                    all_labels.insert(label.to_string());
                                }
                            }
                        }
                        break;
                    }
                }
            }
        }
    }
    
    // 转换为有序列表
    let mut labels: Vec<String> = all_labels.into_iter().collect();
    labels.sort();
    
    Ok(labels)
}

/// 检查 classes.txt 是否存在
#[tauri::command]
pub async fn classes_file_exists(image_folder: String) -> Result<bool, AppError> {
    let classes_path = get_classes_file_path(&image_folder);
    Ok(classes_path.exists())
}
