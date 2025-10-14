use serde::Serialize;

/// 应用错误类型
#[derive(Debug, Serialize)]
pub struct AppError {
    pub message: String,
}

impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        AppError {
            message: format!("IO Error: {}", error),
        }
    }
}

impl From<String> for AppError {
    fn from(error: String) -> Self {
        AppError { message: error }
    }
}

impl From<serde_json::Error> for AppError {
    fn from(error: serde_json::Error) -> Self {
        AppError {
            message: format!("JSON Error: {}", error),
        }
    }
}

impl From<image::ImageError> for AppError {
    fn from(error: image::ImageError) -> Self {
        AppError {
            message: format!("Image Error: {}", error),
        }
    }
}
