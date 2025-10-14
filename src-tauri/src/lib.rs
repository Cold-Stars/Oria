// 模块声明
mod core; // 核心功能（标注、缓存、图片）
mod error; // 错误处理
mod export; // 导出功能（COCO、YOLO）
mod inference; // 推理功能（API、ONNX）
mod models; // 数据模型
mod ui; // UI交互（文件对话框、图片加载）
mod utils; // 工具函数

// 重新导出常用模块
use core::{annotation, cache, image};
use inference::{api_client, model_inference};
use ui::{file_dialog, image_loader};

// 导入Tauri命令需要的类型
use annotation::{load_annotations, save_annotations};
use export::coco::{export_coco, import_coco};
use export::yolo::{export_yolo, export_yolo_batch};
use file_dialog::{open_folder, select_coco_file, select_save_folder};
use image::load_image;
use image_loader::{
    clear_image_cache, generate_thumbnails, get_cache_stats, get_thumbnail, load_image_cached,
    preload_images,
};
use model_inference::{InferenceConfig, InferenceManager};

// 旧的greet函数，保持兼容性
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// 模型推理相关命令

/// 检查API服务健康状态
#[tauri::command]
async fn check_api_health(base_url: String) -> Result<bool, String> {
    let client = api_client::ApiClient::new(base_url);
    client.check_health().await.map_err(|e| e.to_string())
}

/// 获取API模型信息
#[tauri::command]
async fn get_api_model_info(base_url: String) -> Result<api_client::ModelInfo, String> {
    let client = api_client::ApiClient::new(base_url);
    client.get_model_info().await.map_err(|e| e.to_string())
}

/// 对单张图片进行推理
#[tauri::command]
async fn inference_single(
    image_path: String,
    config: InferenceConfig,
) -> Result<model_inference::InferenceResult, String> {
    let manager = InferenceManager::new(config);
    manager
        .inference_single(&image_path)
        .await
        .map_err(|e| e.to_string())
}

/// 批量推理
#[tauri::command]
async fn inference_batch(
    image_paths: Vec<String>,
    start_index: usize,
    count: usize,
    config: InferenceConfig,
) -> Result<model_inference::BatchInferenceResult, String> {
    let manager = InferenceManager::new(config);
    manager
        .inference_batch(image_paths, start_index, count)
        .await
        .map_err(|e| e.to_string())
}

/// 选择ONNX模型文件
#[tauri::command]
async fn select_onnx_model() -> Result<String, String> {
    use rfd::AsyncFileDialog;

    let file = AsyncFileDialog::new()
        .add_filter("ONNX模型", &["onnx"])
        .set_title("选择ONNX模型文件")
        .pick_file()
        .await;

    match file {
        Some(file) => {
            let path = file.path().to_string_lossy().to_string();
            Ok(path)
        }
        None => Err("未选择文件".to_string()),
    }
}

/// 验证ONNX模型（获取模型信息）
#[cfg(feature = "onnx")]
#[tauri::command]
async fn validate_onnx_model(model_path: String) -> Result<OnnxModelInfo, String> {
    use crate::inference::onnx_inference::OnnxInferenceEngine;

    let engine = OnnxInferenceEngine::new(&model_path, 0.25, 0.45, false)
        .map_err(|e| format!("无法加载模型: {}", e))?;

    let (width, height) = engine.get_input_size();
    let class_names = engine.get_class_names().to_vec();

    Ok(OnnxModelInfo {
        model_path,
        input_size: vec![width, height],
        class_names,
    })
}

#[cfg(not(feature = "onnx"))]
#[tauri::command]
async fn validate_onnx_model(_model_path: String) -> Result<OnnxModelInfo, String> {
    Err("ONNX功能未启用，请使用--features onnx编译".to_string())
}

/// ONNX模型信息
#[derive(serde::Serialize)]
struct OnnxModelInfo {
    model_path: String,
    input_size: Vec<u32>,
    class_names: Vec<String>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            open_folder,
            select_save_folder,
            select_coco_file,
            load_image,
            load_image_cached,
            get_thumbnail,
            generate_thumbnails,
            preload_images,
            clear_image_cache,
            get_cache_stats,
            save_annotations,
            load_annotations,
            export_yolo,
            export_yolo_batch,
            export_coco,
            import_coco,
            check_api_health,
            get_api_model_info,
            inference_single,
            inference_batch,
            select_onnx_model,
            validate_onnx_model
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
