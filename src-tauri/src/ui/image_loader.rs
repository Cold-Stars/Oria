use image::GenericImageView;
use serde::Serialize;
use tauri::Emitter;

use crate::cache::IMAGE_CACHE;
use crate::error::AppError;
use crate::models::ImageData;

/// 加载图片（使用缓存）
#[tauri::command]
pub async fn load_image_cached(path: String) -> Result<ImageData, AppError> {
    // 使用 tokio spawn_blocking 避免阻塞（图片解码是 CPU 密集操作）
    let path_clone = path.clone();
    let (width, height) = tokio::task::spawn_blocking(move || {
        let img = image::open(&path_clone)?;
        Ok::<_, AppError>((img.dimensions().0, img.dimensions().1))
    })
    .await
    .map_err(|e| AppError {
        message: format!("Failed to read image dimensions: {}", e),
    })??;

    // 直接加载完整图
    let url = IMAGE_CACHE.get_full_image(&path)?;

    Ok(ImageData {
        url,
        width,
        height,
        file_path: path,
    })
}

/// 获取单个缩略图（快速返回，用于按需加载）
#[tauri::command]
pub async fn get_thumbnail(path: String, size: u32) -> Result<String, AppError> {
    IMAGE_CACHE.get_thumbnail(&path, size)
}

/// 批量生成缩略图（流式，边生成边返回，按顺序生成）
#[tauri::command]
pub async fn generate_thumbnails(
    app_handle: tauri::AppHandle,
    paths: Vec<String>,
    size: u32,
) -> Result<(), AppError> {
    // 使用 tokio spawn 异步生成（不阻塞前端）
    tokio::spawn(async move {
        // 按顺序生成缩略图（不使用并行，保证顺序）
        for (index, path) in paths.iter().enumerate() {
            match IMAGE_CACHE.get_thumbnail(path, size) {
                Ok(data) => {
                    // 实时发送事件到前端
                    let _ = app_handle.emit(
                        "thumbnail-ready",
                        ThumbnailEvent {
                            path: path.clone(),
                            data,
                            index, // 发送索引，前端可以按顺序显示
                        },
                    );
                }
                Err(e) => {
                    println!("Failed to generate thumbnail for {}: {}", path, e.message);
                }
            }
        }

        // 完成后发送完成事件
        let _ = app_handle.emit("thumbnails-complete", ());
    });

    Ok(())
}

#[derive(Clone, serde::Serialize)]
struct ThumbnailEvent {
    path: String,
    data: String,
    index: usize,
}

/// 预加载图片
#[tauri::command]
pub async fn preload_images(
    current_index: usize,
    all_paths: Vec<String>,
    preload_count: usize,
) -> Result<(), AppError> {
    // 计算需要预加载的图片索引范围
    let total = all_paths.len();
    let mut paths_to_preload = Vec::new();

    // 预加载当前图片前后各 preload_count 张
    for i in 1..=preload_count {
        // 后面的图片（优先，因为用户通常向后浏览）
        if current_index + i < total {
            paths_to_preload.push(all_paths[current_index + i].clone());
        }
        // 前面的图片
        if current_index >= i {
            paths_to_preload.push(all_paths[current_index - i].clone());
        }
    }

    if paths_to_preload.is_empty() {
        return Ok(());
    }

    // 在后台异步预加载（不阻塞）
    tokio::spawn(async move {
        for path in paths_to_preload {
            let _ = IMAGE_CACHE.get_full_image(&path);
        }
    });

    Ok(())
}

/// 清空缓存
#[tauri::command]
pub async fn clear_image_cache() -> Result<(), AppError> {
    IMAGE_CACHE.clear();
    Ok(())
}

/// 获取缓存统计信息
#[tauri::command]
pub async fn get_cache_stats() -> Result<CacheStatsData, AppError> {
    let stats = IMAGE_CACHE.stats();
    Ok(CacheStatsData {
        full_cache_count: stats.full_cache_count,
        thumbnail_cache_count: stats.thumbnail_cache_count,
        full_cache_max: stats.full_cache_max,
        thumbnail_cache_max: stats.thumbnail_cache_max,
    })
}

#[derive(Debug, Serialize)]
pub struct CacheStatsData {
    full_cache_count: usize,
    thumbnail_cache_count: usize,
    full_cache_max: usize,
    thumbnail_cache_max: usize,
}
