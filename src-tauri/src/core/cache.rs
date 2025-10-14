use base64::{engine::general_purpose, Engine as _};
use image::{imageops::FilterType, GenericImageView};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::error::AppError;

/// 图片缓存项
#[derive(Clone)]
struct CacheItem {
    data: String,   // base64 编码的图片数据
    timestamp: u64, // 缓存时间戳
}

/// 图片缓存管理器
pub struct ImageCache {
    // 完整图片缓存
    full_cache: Arc<Mutex<HashMap<String, CacheItem>>>,
    // 缩略图缓存
    thumbnail_cache: Arc<Mutex<HashMap<String, CacheItem>>>,
    // 最大缓存数量
    max_full_cache: usize,
    max_thumbnail_cache: usize,
}

impl ImageCache {
    pub fn new(max_full_cache: usize, max_thumbnail_cache: usize) -> Self {
        Self {
            full_cache: Arc::new(Mutex::new(HashMap::new())),
            thumbnail_cache: Arc::new(Mutex::new(HashMap::new())),
            max_full_cache,
            max_thumbnail_cache,
        }
    }

    /// 获取完整图片（从缓存或加载）
    pub fn get_full_image(&self, path: &str) -> Result<String, AppError> {
        // 先检查缓存
        {
            let cache = self.full_cache.lock().unwrap();
            if let Some(item) = cache.get(path) {
                // println!("✅ Cache hit for full image: {}", path);
                return Ok(item.data.clone());
            }
        }

        // println!("❌ Cache miss for full image: {}", path);
        // 缓存未命中，加载图片
        let data = self.load_full_image(path)?;

        // 存入缓存
        self.cache_full_image(path.to_string(), data.clone());

        Ok(data)
    }

    /// 获取缩略图（从缓存或生成）
    pub fn get_thumbnail(&self, path: &str, size: u32) -> Result<String, AppError> {
        let cache_key = format!("{}_{}", path, size);

        // 先检查缓存
        {
            let cache = self.thumbnail_cache.lock().unwrap();
            if let Some(item) = cache.get(&cache_key) {
                // println!("✅ Cache hit for thumbnail: {}", path);
                return Ok(item.data.clone());
            }
        }

        // println!("❌ Cache miss for thumbnail: {}", path);
        // 缓存未命中，生成缩略图
        let data = self.generate_thumbnail(path, size)?;

        // 存入缓存
        self.cache_thumbnail(cache_key, data.clone());

        Ok(data)
    }

    /// 加载完整图片
    fn load_full_image(&self, path: &str) -> Result<String, AppError> {
        let image_path = Path::new(path);
        if !image_path.exists() {
            return Err(AppError {
                message: format!("Image file does not exist: {}", path),
            });
        }

        let image_bytes = std::fs::read(path)?;
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

        Ok(format!("data:{};base64,{}", mime_type, base64_data))
    }

    /// 生成缩略图
    fn generate_thumbnail(&self, path: &str, size: u32) -> Result<String, AppError> {
        let image_path = Path::new(path);
        if !image_path.exists() {
            return Err(AppError {
                message: format!("Image file does not exist: {}", path),
            });
        }

        // 读取图片
        let img = image::open(path)?;
        let (width, height) = img.dimensions();

        // 计算缩略图尺寸（保持宽高比）
        let (thumb_width, thumb_height) = if width > height {
            let ratio = height as f32 / width as f32;
            (size, (size as f32 * ratio) as u32)
        } else {
            let ratio = width as f32 / height as f32;
            ((size as f32 * ratio) as u32, size)
        };

        // 生成缩略图
        let thumbnail = img.resize(thumb_width, thumb_height, FilterType::Lanczos3);

        // 编码为 JPEG（缩略图使用 JPEG 以减小大小）
        let mut buffer = std::io::Cursor::new(Vec::new());
        thumbnail.write_to(&mut buffer, image::ImageFormat::Jpeg)?;
        let jpeg_bytes = buffer.into_inner();

        let base64_data = general_purpose::STANDARD.encode(&jpeg_bytes);
        Ok(format!("data:image/jpeg;base64,{}", base64_data))
    }

    /// 缓存完整图片
    fn cache_full_image(&self, path: String, data: String) {
        let mut cache = self.full_cache.lock().unwrap();

        // 如果缓存已满，移除最旧的项
        if cache.len() >= self.max_full_cache {
            if let Some(oldest_key) = cache
                .iter()
                .min_by_key(|(_, item)| item.timestamp)
                .map(|(key, _)| key.clone())
            {
                cache.remove(&oldest_key);
                // println!("🗑️  Evicted old cache entry: {}", oldest_key);
            }
        }

        cache.insert(
            path,
            CacheItem {
                data,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            },
        );
    }

    /// 缓存缩略图
    fn cache_thumbnail(&self, cache_key: String, data: String) {
        let mut cache = self.thumbnail_cache.lock().unwrap();

        // 如果缓存已满，移除最旧的项
        if cache.len() >= self.max_thumbnail_cache {
            if let Some(oldest_key) = cache
                .iter()
                .min_by_key(|(_, item)| item.timestamp)
                .map(|(key, _)| key.clone())
            {
                cache.remove(&oldest_key);
            }
        }

        cache.insert(
            cache_key,
            CacheItem {
                data,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            },
        );
    }

    /// 清空缓存
    pub fn clear(&self) {
        self.full_cache.lock().unwrap().clear();
        self.thumbnail_cache.lock().unwrap().clear();
    }

    /// 获取缓存统计信息
    pub fn stats(&self) -> CacheStats {
        let full_count = self.full_cache.lock().unwrap().len();
        let thumbnail_count = self.thumbnail_cache.lock().unwrap().len();

        CacheStats {
            full_cache_count: full_count,
            thumbnail_cache_count: thumbnail_count,
            full_cache_max: self.max_full_cache,
            thumbnail_cache_max: self.max_thumbnail_cache,
        }
    }
}

#[derive(Debug)]
pub struct CacheStats {
    pub full_cache_count: usize,
    pub thumbnail_cache_count: usize,
    pub full_cache_max: usize,
    pub thumbnail_cache_max: usize,
}

// 全局缓存实例
// 参数：完整图片(10), 缩略图(200)
lazy_static::lazy_static! {
    pub static ref IMAGE_CACHE: ImageCache = ImageCache::new(10, 200);
}
