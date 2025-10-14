use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// API推理请求
#[derive(Debug, Serialize)]
pub struct InferenceRequest {
    pub image_base64: String,
    pub conf_threshold: f32,
    pub iou_threshold: f32,
}

/// API推理响应 - 单个检测框
#[derive(Debug, Deserialize, Clone)]
pub struct Detection {
    pub class_id: usize,
    pub class_name: String,
    pub confidence: f32,
    pub bbox: Vec<f32>, // [x_min, y_min, x_max, y_max] or [cx, cy, w, h] for rotated bbox
    #[serde(skip_serializing_if = "Option::is_none")]
    pub angle: Option<f32>, // 旋转角度(度)，仅旋转框使用
}

/// API推理响应
#[derive(Debug, Deserialize)]
pub struct InferenceResponse {
    pub detections: Vec<Detection>,
    pub inference_time_ms: f32,
}

/// API客户端
pub struct ApiClient {
    base_url: String,
    client: reqwest::Client,
}

impl ApiClient {
    /// 创建新的API客户端
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::Client::new(),
        }
    }

    /// 检查API服务是否可用
    pub async fn check_health(&self) -> Result<bool> {
        let url = format!("{}/health", self.base_url);
        let response = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
            .context("无法连接到API服务")?;

        Ok(response.status().is_success())
    }

    /// 推理单张图片
    pub async fn predict(
        &self,
        image_base64: String,
        conf_threshold: f32,
        iou_threshold: f32,
    ) -> Result<InferenceResponse> {
        let url = format!("{}/predict", self.base_url);
        let request = InferenceRequest {
            image_base64,
            conf_threshold,
            iou_threshold,
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
            .context("API请求失败")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("API返回错误 {}: {}", status, error_text);
        }

        let result = response
            .json::<InferenceResponse>()
            .await
            .context("解析API响应失败")?;

        Ok(result)
    }

    /// 获取模型信息
    pub async fn get_model_info(&self) -> Result<ModelInfo> {
        let url = format!("{}/model_info", self.base_url);
        let response = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
            .context("获取模型信息失败")?;

        let info = response
            .json::<ModelInfo>()
            .await
            .context("解析模型信息失败")?;

        Ok(info)
    }
}

/// 模型信息
#[derive(Debug, Deserialize, Serialize)]
pub struct ModelInfo {
    pub model_name: String,
    pub class_names: Vec<String>,
    pub input_size: Vec<u32>, // [width, height]
}
