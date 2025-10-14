use anyhow::{Context, Result};
use base64::{engine::general_purpose, Engine as _};
use image::{DynamicImage, GenericImageView};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use super::api_client::{ApiClient, Detection};
use crate::models::AnnotationData;

#[cfg(feature = "onnx")]
use lazy_static::lazy_static;

// ONNX 推理引擎缓存
#[cfg(feature = "onnx")]
lazy_static! {
    static ref ONNX_ENGINE_CACHE: Mutex<HashMap<String, Arc<super::onnx_inference::OnnxInferenceEngine>>> =
        Mutex::new(HashMap::new());
}

/// 推理模式
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InferenceMode {
    /// 使用API推理
    Api {
        base_url: String,
        conf_threshold: f32,
        iou_threshold: f32,
    },
    /// 使用ONNX推理 (需要feature)
    #[cfg(feature = "onnx")]
    Onnx {
        model_path: String,
        conf_threshold: f32,
        iou_threshold: f32,
        use_gpu: bool,
    },
}

/// 推理配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceConfig {
    pub mode: InferenceMode,
    pub count: InferenceCount,
}

/// 推理数量配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InferenceCount {
    /// 推理指定数量
    Count { value: usize },
    /// 推理全部
    All,
}

/// 推理结果
#[derive(Debug, Serialize)]
pub struct InferenceResult {
    pub image_path: String,
    pub annotations: Vec<AnnotationData>,
    pub inference_time_ms: f32,
}

/// 批量推理结果
#[derive(Debug, Serialize)]
pub struct BatchInferenceResult {
    pub results: Vec<InferenceResult>,
    pub total_time_ms: f32,
    pub success_count: usize,
    pub error_count: usize,
}

/// 模型推理管理器
pub struct InferenceManager {
    config: InferenceConfig,
}

impl InferenceManager {
    pub fn new(config: InferenceConfig) -> Self {
        Self { config }
    }

    /// 推理单张图片
    pub async fn inference_single(&self, image_path: &str) -> Result<InferenceResult> {
        // 加载图片
        let img = image::open(image_path).context("无法打开图片")?;
        let (width, height) = img.dimensions();

        // 根据模式选择推理方式
        let (detections, inference_time) = match &self.config.mode {
            InferenceMode::Api {
                base_url,
                conf_threshold,
                iou_threshold,
            } => {
                self.inference_with_api(&img, base_url, *conf_threshold, *iou_threshold)
                    .await?
            }
            #[cfg(feature = "onnx")]
            InferenceMode::Onnx {
                model_path,
                conf_threshold,
                iou_threshold,
                use_gpu,
            } => self.inference_with_onnx(
                &img,
                model_path,
                *conf_threshold,
                *iou_threshold,
                *use_gpu,
            )?,
        };

        // 转换为标注数据
        let annotations = detections
            .into_iter()
            .map(|det| self.detection_to_annotation(det, width, height))
            .collect::<Vec<_>>();

        // println!(
        //     "推理完成: {} - 检测到 {} 个目标",
        //     image_path,
        //     annotations.len()
        // );

        // 保存标注到文件
        crate::core::annotation::save_annotations_internal(image_path, annotations.clone())
            .map_err(|e| anyhow::anyhow!("无法保存标注: {}", e.message))?;

        // println!("标注已保存: {}", image_path);

        Ok(InferenceResult {
            image_path: image_path.to_string(),
            annotations,
            inference_time_ms: inference_time,
        })
    }

    /// 批量推理
    pub async fn inference_batch(
        &self,
        image_paths: Vec<String>,
        start_index: usize,
        count: usize,
    ) -> Result<BatchInferenceResult> {
        let start_time = std::time::Instant::now();
        let mut results = Vec::new();
        let mut success_count = 0;
        let mut error_count = 0;

        // 计算要推理的图片范围
        let end_index = (start_index + count).min(image_paths.len());
        let paths_to_infer = &image_paths[start_index..end_index];

        for path in paths_to_infer {
            match self.inference_single(path).await {
                Ok(result) => {
                    results.push(result);
                    success_count += 1;
                }
                Err(e) => {
                    eprintln!("推理失败 {}: {}", path, e);
                    error_count += 1;
                }
            }
        }

        let total_time = start_time.elapsed().as_millis() as f32;

        Ok(BatchInferenceResult {
            results,
            total_time_ms: total_time,
            success_count,
            error_count,
        })
    }

    /// 使用API推理
    async fn inference_with_api(
        &self,
        img: &DynamicImage,
        base_url: &str,
        conf_threshold: f32,
        iou_threshold: f32,
    ) -> Result<(Vec<Detection>, f32)> {
        // 转换图片为base64
        let image_base64 = self.image_to_base64(img)?;

        // 创建API客户端并推理
        let client = ApiClient::new(base_url.to_string());
        let response = client
            .predict(image_base64, conf_threshold, iou_threshold)
            .await?;

        Ok((response.detections, response.inference_time_ms))
    }

    /// 使用ONNX推理 (需要feature)
    #[cfg(feature = "onnx")]
    fn inference_with_onnx(
        &self,
        img: &DynamicImage,
        model_path: &str,
        conf_threshold: f32,
        iou_threshold: f32,
        use_gpu: bool,
    ) -> Result<(Vec<Detection>, f32)> {
        use super::onnx_inference::OnnxInferenceEngine;

        // 生成缓存key：包含模型路径和GPU配置
        let cache_key = format!("{}:{}:{}", model_path, use_gpu, conf_threshold);

        // 尝试从缓存获取引擎
        let engine = {
            let mut cache = ONNX_ENGINE_CACHE.lock().unwrap();

            if let Some(cached_engine) = cache.get(&cache_key) {
                // println!("♻️  复用已加载的ONNX推理引擎");
                Arc::clone(cached_engine)
            } else {
                // println!("🆕 创建新的ONNX推理引擎并缓存");
                let new_engine = Arc::new(
                    OnnxInferenceEngine::new(model_path, conf_threshold, iou_threshold, use_gpu)
                        .context("无法创建ONNX推理器")?,
                );
                cache.insert(cache_key.clone(), Arc::clone(&new_engine));
                new_engine
            }
        };

        // 推理（只计时实际推理部分）
        let start_time = std::time::Instant::now();
        let detections = engine.inference(img).context("ONNX推理失败")?;
        let inference_time = start_time.elapsed().as_secs_f32() * 1000.0;

        Ok((detections, inference_time))
    }

    /// 将图片转换为base64
    fn image_to_base64(&self, img: &DynamicImage) -> Result<String> {
        let mut buffer = Vec::new();
        img.write_to(
            &mut std::io::Cursor::new(&mut buffer),
            image::ImageOutputFormat::Jpeg(90),
        )
        .context("无法编码图片")?;

        Ok(general_purpose::STANDARD.encode(&buffer))
    }

    /// 将检测结果转换为标注数据
    fn detection_to_annotation(
        &self,
        detection: Detection,
        img_width: u32,
        img_height: u32,
    ) -> AnnotationData {
        if let Some(angle_deg) = detection.angle {
            // 旋转框：bbox格式为[cx, cy, w, h]
            let cx = detection.bbox[0].max(0.0).min(img_width as f32);
            let cy = detection.bbox[1].max(0.0).min(img_height as f32);
            let width = detection.bbox[2].max(1.0);
            let height = detection.bbox[3].max(1.0);

            // 前端期望：
            // 1. x, y 是左上角坐标（不是中心点）
            // 2. rotation 是弧度值（不是度）
            let x_left_top = cx - width / 2.0;
            let y_left_top = cy - height / 2.0;
            let angle_rad = angle_deg * std::f32::consts::PI / 180.0;

            // 创建旋转矩形标注
            AnnotationData {
                id: uuid::Uuid::new_v4().to_string(),
                annotation_type: "rotated-rectangle".to_string(),
                x: x_left_top as f64,
                y: y_left_top as f64,
                width: width as f64,
                height: height as f64,
                rotation: Some(angle_rad as f64),
                label: detection.class_name,
                created: chrono::Utc::now().to_rfc3339(),
                visible: true,
            }
        } else {
            // 普通框：bbox格式为[x_min, y_min, x_max, y_max]
            let x_min = detection.bbox[0].max(0.0).min(img_width as f32);
            let y_min = detection.bbox[1].max(0.0).min(img_height as f32);
            let x_max = detection.bbox[2].max(0.0).min(img_width as f32);
            let y_max = detection.bbox[3].max(0.0).min(img_height as f32);

            let width = (x_max - x_min).max(1.0);
            let height = (y_max - y_min).max(1.0);

            // 创建普通矩形标注
            // 注意：标注系统中，普通矩形的x,y是左上角坐标
            AnnotationData {
                id: uuid::Uuid::new_v4().to_string(),
                annotation_type: "rectangle".to_string(),
                x: x_min as f64,
                y: y_min as f64,
                width: width as f64,
                height: height as f64,
                rotation: None,
                label: detection.class_name,
                created: chrono::Utc::now().to_rfc3339(),
                visible: true,
            }
        }
    }
}
