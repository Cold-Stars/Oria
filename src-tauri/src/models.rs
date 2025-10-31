use serde::{Deserialize, Serialize};

/// 图片数据结构
#[derive(Debug, Serialize)]
pub struct ImageData {
    pub url: String,
    pub width: u32,  // 图片宽度
    pub height: u32, // 图片高度
    pub file_path: String,
}

/// 标注数据结构
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnnotationData {
    pub id: String,
    #[serde(rename = "type")]
    pub annotation_type: String, // "rectangle" or "rotated-rectangle"
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation: Option<f64>, // 旋转角度，仅旋转矩形使用
    pub label: String,
    #[serde(skip_serializing, default)]
    pub created: String, // ISO时间戳（不保存到JSON）
    #[serde(skip_serializing, default = "default_visible")]
    pub visible: bool, // 是否可见（不保存到JSON）
}

/// 默认可见
fn default_visible() -> bool {
    true
}

/// 标注文件数据结构
#[derive(Debug, Serialize, Deserialize)]
pub struct AnnotationFile {
    pub version: String,
    pub image_path: String,
    pub image_width: u32,
    pub image_height: u32,
    pub annotations: Vec<AnnotationData>,
    pub created: String,
    pub modified: String,
}
