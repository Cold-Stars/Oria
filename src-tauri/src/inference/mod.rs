//! 推理模块
//!
//! 包含API推理和ONNX推理相关功能

pub mod api_client;
pub mod model_inference;

#[cfg(feature = "onnx")]
pub mod onnx_inference;

