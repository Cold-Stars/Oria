use crate::error::AppError;
use crate::utils::is_image_file;

/// 选择保存文件夹
#[tauri::command]
pub async fn select_save_folder(_window: tauri::Window) -> Result<String, AppError> {
    let folder_path = rfd::AsyncFileDialog::new()
        .set_title("选择导出文件夹")
        .pick_folder()
        .await;

    let folder_path = folder_path.ok_or_else(|| AppError {
        message: "No folder selected".to_string(),
    })?;

    let folder_path_str = folder_path.path().to_string_lossy().to_string();
    // println!("Selected save folder: {}", folder_path_str);

    Ok(folder_path_str)
}

/// 打开文件夹并获取图片文件列表
#[tauri::command]
pub async fn open_folder(_window: tauri::Window) -> Result<Vec<String>, AppError> {
    let folder_path = rfd::AsyncFileDialog::new()
        .set_title("选择包含图片的文件夹")
        .pick_folder()
        .await;

    let folder_path = folder_path.ok_or_else(|| AppError {
        message: "No folder selected".to_string(),
    })?;

    let folder_path = folder_path.path();
    // println!("Selected folder: {:?}", folder_path);

    let mut image_files = Vec::new();
    let entries = std::fs::read_dir(folder_path)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && is_image_file(&path) {
            if let Some(path_str) = path.to_str() {
                image_files.push(path_str.to_string());
                // println!("Found image: {}", path_str);
            }
        }
    }

    // 按文件名自然排序
    image_files.sort_by(|a, b| natord::compare(a, b));

    // println!("Total {} image files found", image_files.len());
    Ok(image_files)
}

/// 选择COCO JSON文件
#[tauri::command]
pub async fn select_coco_file(_window: tauri::Window) -> Result<String, AppError> {
    let file_path = rfd::AsyncFileDialog::new()
        .set_title("选择COCO格式的JSON文件")
        .add_filter("JSON文件", &["json"])
        .pick_file()
        .await;

    let file_path = file_path.ok_or_else(|| AppError {
        message: "No file selected".to_string(),
    })?;

    let file_path_str = file_path.path().to_string_lossy().to_string();
    // println!("Selected COCO file: {}", file_path_str);

    Ok(file_path_str)
}
