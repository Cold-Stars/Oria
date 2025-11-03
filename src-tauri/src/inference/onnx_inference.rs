/// ONNX推理模块
/// 基于YOLOv8-ONNXRuntime-Rust简化实现
use anyhow::{Context, Result};
use image::{DynamicImage, GenericImageView};
use ndarray::{s, Array, ArrayView, Axis, CowArray, IxDyn};
use ort::{Environment, ExecutionProvider, GraphOptimizationLevel, Session, SessionBuilder, Value};
use std::sync::Arc;

use super::api_client::Detection;

/// ONNX推理器
pub struct OnnxInferenceEngine {
    session: Arc<std::sync::Mutex<Session>>,
    #[allow(dead_code)]
    environment: Arc<Environment>,
    input_width: u32,
    input_height: u32,
    conf_threshold: f32,
    iou_threshold: f32,
    class_names: Vec<String>,
    #[allow(dead_code)]
    use_gpu: bool,
}

impl OnnxInferenceEngine {
    /// 创建新的ONNX推理器
    pub fn new(
        model_path: &str,
        conf_threshold: f32,
        iou_threshold: f32,
        use_gpu: bool,
    ) -> Result<Self> {
        // 创建ONNX Runtime环境，根据参数选择执行提供者
        let execution_providers = if use_gpu {
            // println!("尝试使用 GPU 加速 (CUDA)");
            vec![
                ExecutionProvider::CUDA(Default::default()),
                ExecutionProvider::CPU(Default::default()),
            ]
        } else {
            // println!("使用 CPU 推理");
            vec![ExecutionProvider::CPU(Default::default())]
        };

        let environment = Arc::new(
            Environment::builder()
                .with_name("yolov8")
                .with_execution_providers(execution_providers)
                .build()
                .context("无法创建ONNX Runtime环境")?,
        );

        // 加载ONNX模型
        let session = SessionBuilder::new(&environment)?
            .with_optimization_level(GraphOptimizationLevel::Level1)?
            .with_intra_threads(4)?
            .with_model_from_file(model_path)
            .context("无法加载ONNX模型")?;

        // 检查实际使用的执行提供者
        // println!("✅ ONNX模型加载成功！");

        // 获取输入尺寸 - 从输入形状获取
        let input_shape = session.inputs[0].dimensions().collect::<Vec<_>>();
        let input_height = input_shape
            .get(2)
            .and_then(|d| d.map(|v| v as u32))
            .unwrap_or(640);
        let input_width = input_shape
            .get(3)
            .and_then(|d| d.map(|v| v as u32))
            .unwrap_or(640);

        // 尝试从元数据获取类别名称
        let class_names = Self::get_class_names_from_metadata(&session);

        Ok(Self {
            session: Arc::new(std::sync::Mutex::new(session)),
            environment,
            input_width,
            input_height,
            conf_threshold,
            iou_threshold,
            class_names,
            use_gpu,
        })
    }

    /// 从模型元数据获取类别名称 (静态方法)
    fn get_class_names_from_metadata(session: &Session) -> Vec<String> {
        // 尝试从元数据读取类别名称
        if let Ok(metadata) = session.metadata() {
            if let Ok(Some(names_str)) = metadata.custom("names") {
                // 解析格式: {0: 'person', 1: 'bicycle', ...}
                let re = regex::Regex::new(r#"['"]([^'"]+)['"]"#).unwrap();
                let names: Vec<String> = re
                    .captures_iter(names_str.as_ref())
                    .map(|cap| cap[1].to_string())
                    .collect();
                if !names.is_empty() {
                    return names;
                }
            }
        }

        // 默认使用COCO类别
        vec![
            "person",
            "bicycle",
            "car",
            "motorcycle",
            "airplane",
            "bus",
            "train",
            "truck",
            "boat",
            "traffic light",
            "fire hydrant",
            "stop sign",
            "parking meter",
            "bench",
            "bird",
            "cat",
            "dog",
            "horse",
            "sheep",
            "cow",
            "elephant",
            "bear",
            "zebra",
            "giraffe",
            "backpack",
            "umbrella",
            "handbag",
            "tie",
            "suitcase",
            "frisbee",
            "skis",
            "snowboard",
            "sports ball",
            "kite",
            "baseball bat",
            "baseball glove",
            "skateboard",
            "surfboard",
            "tennis racket",
            "bottle",
            "wine glass",
            "cup",
            "fork",
            "knife",
            "spoon",
            "bowl",
            "banana",
            "apple",
            "sandwich",
            "orange",
            "broccoli",
            "carrot",
            "hot dog",
            "pizza",
            "donut",
            "cake",
            "chair",
            "couch",
            "potted plant",
            "bed",
            "dining table",
            "toilet",
            "tv",
            "laptop",
            "mouse",
            "remote",
            "keyboard",
            "cell phone",
            "microwave",
            "oven",
            "toaster",
            "sink",
            "refrigerator",
            "book",
            "clock",
            "vase",
            "scissors",
            "teddy bear",
            "hair drier",
            "toothbrush",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect()
    }

    /// 推理单张图片
    pub fn inference(&self, image: &DynamicImage) -> Result<Vec<Detection>> {
        let start_time = std::time::Instant::now();

        // println!(
        //     "🚀 开始推理 [设备: {}]",
        //     if self.use_gpu { "GPU (CUDA)" } else { "CPU" }
        // );

        // 预处理
        let preprocess_start = std::time::Instant::now();
        let (preprocessed, ratio, padding) = self.preprocess(image)?;
        let _preprocess_time = preprocess_start.elapsed().as_secs_f32() * 1000.0;

        // 推理 - 创建 Value
        let inference_start = std::time::Instant::now();
        let session_lock = self.session.lock().unwrap();
        let allocator = session_lock.allocator();

        // 转换为 CowArray 用于创建 Value
        let array_view: ArrayView<f32, IxDyn> = preprocessed.view();
        let cow_array: CowArray<f32, IxDyn> = CowArray::from(array_view);
        let input_tensor = Value::from_array(allocator, &cow_array)?;

        let outputs = session_lock
            .run(vec![input_tensor])
            .context("ONNX推理失败")?;
        let _inference_time = inference_start.elapsed().as_secs_f32() * 1000.0;

        // 后处理
        let postprocess_start = std::time::Instant::now();
        let detections = self.postprocess(&outputs, image, ratio, padding)?;
        let _postprocess_time = postprocess_start.elapsed().as_secs_f32() * 1000.0;

        let _total_time = start_time.elapsed().as_secs_f32() * 1000.0;

        // println!(
        //     "⏱️  推理性能统计:\n   预处理: {:.2}ms | 模型推理: {:.2}ms | 后处理: {:.2}ms | 总计: {:.2}ms",
        //     preprocess_time, inference_time, postprocess_time, total_time
        // );

        Ok(detections)
    }

    /// 图片预处理
    /// 返回：(预处理后的数组, 缩放比例, (padding_left, padding_top))
    fn preprocess(&self, image: &DynamicImage) -> Result<(Array<f32, IxDyn>, f32, (f32, f32))> {
        let (img_width, img_height) = image.dimensions();
        let img_width = img_width as f32;
        let img_height = img_height as f32;

        // 计算缩放比例（保持宽高比）
        let ratio =
            (self.input_width as f32 / img_width).min(self.input_height as f32 / img_height);

        let new_width = (img_width * ratio).round() as u32;
        let new_height = (img_height * ratio).round() as u32;

        // 计算padding（与Python letterbox保持一致）
        let dw = (self.input_width as f32 - new_width as f32) / 2.0;
        let dh = (self.input_height as f32 - new_height as f32) / 2.0;

        // Python代码中的padding计算：top, bottom = int(round(dh - 0.1)), int(round(dh + 0.1))
        let pad_left = (dw - 0.1).round() as usize;
        let pad_top = (dh - 0.1).round() as usize;

        // 缩放图片
        let resized =
            image.resize_exact(new_width, new_height, image::imageops::FilterType::Triangle);

        // 创建填充后的数组 (1, 3, H, W)，填充值为 114/255
        let mut input_array =
            Array::ones((1, 3, self.input_height as usize, self.input_width as usize)).into_dyn();
        input_array.fill(114.0 / 255.0);

        // 将缩放后的图片复制到数组中（归一化到[0,1]），考虑padding
        for (x, y, pixel) in resized.to_rgb8().enumerate_pixels() {
            let x = x as usize + pad_left;
            let y = y as usize + pad_top;
            input_array[[0, 0, y, x]] = pixel[0] as f32 / 255.0;
            input_array[[0, 1, y, x]] = pixel[1] as f32 / 255.0;
            input_array[[0, 2, y, x]] = pixel[2] as f32 / 255.0;
        }

        Ok((input_array, ratio, (dw, dh)))
    }

    /// 后处理推理结果
    fn postprocess(
        &self,
        outputs: &[Value],
        original_image: &DynamicImage,
        ratio: f32,
        padding: (f32, f32),
    ) -> Result<Vec<Detection>> {
        // 获取输出张量
        let output_tensor = outputs[0]
            .try_extract::<f32>()
            .context("无法提取输出张量")?;

        let tensor_view = output_tensor.view();
        let shape = tensor_view.shape();
        let data = tensor_view.as_slice().context("无法获取输出数据")?;

        // 将数据转换为ndarray
        let output = Array::from_shape_vec((shape[0], shape[1], shape[2]), data.to_vec())?;

        let (img_width, img_height) = original_image.dimensions();
        let mut detections = Vec::new();

        // output shape: (1, num_features, num_anchors)
        // 移除batch维度
        let output = output.index_axis_move(Axis(0), 0); // shape: (num_features, num_anchors)

        // 判断是否是旋转框模型
        // 普通框: num_features = 4 + num_classes (e.g., 84 for COCO)
        // 旋转框: num_features = 4 + num_classes + 1 (最后一维是角度)
        let num_features = output.shape()[0];
        let num_classes = self.class_names.len();
        let is_rotated = num_features == (4 + num_classes + 1);

        // 转置为 (num_anchors, num_features)
        let output = output.t();

        // 遍历每个anchor
        for row in output.axis_iter(Axis(0)) {
            // 前4个是bbox坐标 (cx, cy, w, h)
            let cx: f32 = row[0];
            let cy: f32 = row[1];
            let w: f32 = row[2];
            let h: f32 = row[3];

            // 提取类别置信度
            let class_scores = if is_rotated {
                // 旋转框: [cx, cy, w, h, class0, class1, ..., classN, angle]
                row.slice(s![4..(4 + num_classes)])
            } else {
                // 普通框: [cx, cy, w, h, class0, class1, ..., classN]
                row.slice(s![4..])
            };

            // 找到最大置信度和对应的类别
            let (class_id, &confidence) = class_scores
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                .unwrap();

            // 置信度过滤
            if confidence < self.conf_threshold {
                continue;
            }

            // 提取角度信息（如果是旋转框）
            let angle = if is_rotated {
                let mut angle_rad = row[num_features - 1]; // 最后一维是角度(弧度)

                // 角度范围处理，参考Python实现
                // if 0.5 * math.pi <= angle <= 0.75 * math.pi: angle -= math.pi
                let pi = std::f32::consts::PI;
                if angle_rad >= 0.5 * pi && angle_rad <= 0.75 * pi {
                    angle_rad -= pi;
                }

                // 转换为度（去掉之前的负号，因为关于y轴对称）
                let mut angle_deg = angle_rad * 180.0 / pi;

                // 确保角度为非负值，将负值转换为 [0, 360) 范围
                if angle_deg < 0.0 {
                    angle_deg += 360.0;
                }

                Some(angle_deg)
            } else {
                None
            };

            // 坐标转换：从模型输出空间转换到原图空间
            // 参考Python的scale_boxes函数：先减padding，再除以ratio
            let (pad_w, pad_h) = padding;
            let cx_img = (cx - pad_w) / ratio;
            let cy_img = (cy - pad_h) / ratio;
            let w_img = w / ratio;
            let h_img = h / ratio;

            // 获取类别名称
            let class_name = self
                .class_names
                .get(class_id)
                .cloned()
                .unwrap_or_else(|| format!("class_{}", class_id));

            if is_rotated {
                // 旋转框：bbox保持为[cx, cy, w, h]格式，用于后续绘制旋转矩形
                detections.push(Detection {
                    class_id,
                    class_name,
                    confidence,
                    bbox: vec![cx_img, cy_img, w_img, h_img],
                    angle,
                });
            } else {
                // 普通框：转换为[x_min, y_min, x_max, y_max]格式
                // 注意：普通框也需要减去padding
                let x_min_raw = cx - w / 2.0;
                let y_min_raw = cy - h / 2.0;
                let x_max_raw = cx + w / 2.0;
                let y_max_raw = cy + h / 2.0;

                let (pad_w, pad_h) = padding;
                let x_min = ((x_min_raw - pad_w) / ratio).max(0.0).min(img_width as f32);
                let y_min = ((y_min_raw - pad_h) / ratio)
                    .max(0.0)
                    .min(img_height as f32);
                let x_max = ((x_max_raw - pad_w) / ratio).max(0.0).min(img_width as f32);
                let y_max = ((y_max_raw - pad_h) / ratio)
                    .max(0.0)
                    .min(img_height as f32);

                detections.push(Detection {
                    class_id,
                    class_name,
                    confidence,
                    bbox: vec![x_min, y_min, x_max, y_max],
                    angle: None,
                });
            }
        }

        // NMS (非极大值抑制)
        let detections = self.non_max_suppression(detections);

        Ok(detections)
    }

    /// 非极大值抑制 (NMS)
    fn non_max_suppression(&self, mut detections: Vec<Detection>) -> Vec<Detection> {
        // 按置信度排序
        detections.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut keep = Vec::new();

        while !detections.is_empty() {
            let current = detections.remove(0);
            keep.push(current.clone());

            detections.retain(|det| {
                // 只对同一类别进行NMS
                if det.class_id != current.class_id {
                    return true;
                }

                // 计算IOU
                let iou = Self::calculate_iou(&current.bbox, &det.bbox, current.angle.is_some());
                iou < self.iou_threshold
            });
        }

        keep
    }

    /// 计算两个bbox的IOU
    /// is_rotated: 如果为true，bbox格式为[cx, cy, w, h]；否则为[x_min, y_min, x_max, y_max]
    fn calculate_iou(box1: &[f32], box2: &[f32], is_rotated: bool) -> f32 {
        let (x1_min, y1_min, x1_max, y1_max) = if is_rotated {
            // 从[cx, cy, w, h]转换为[x_min, y_min, x_max, y_max]
            let cx = box1[0];
            let cy = box1[1];
            let w = box1[2];
            let h = box1[3];
            (cx - w / 2.0, cy - h / 2.0, cx + w / 2.0, cy + h / 2.0)
        } else {
            (box1[0], box1[1], box1[2], box1[3])
        };

        let (x2_min, y2_min, x2_max, y2_max) = if is_rotated {
            // 从[cx, cy, w, h]转换为[x_min, y_min, x_max, y_max]
            let cx = box2[0];
            let cy = box2[1];
            let w = box2[2];
            let h = box2[3];
            (cx - w / 2.0, cy - h / 2.0, cx + w / 2.0, cy + h / 2.0)
        } else {
            (box2[0], box2[1], box2[2], box2[3])
        };

        // 计算交集
        let inter_x_min = x1_min.max(x2_min);
        let inter_y_min = y1_min.max(y2_min);
        let inter_x_max = x1_max.min(x2_max);
        let inter_y_max = y1_max.min(y2_max);

        let inter_width = (inter_x_max - inter_x_min).max(0.0);
        let inter_height = (inter_y_max - inter_y_min).max(0.0);
        let inter_area = inter_width * inter_height;

        // 计算并集
        let box1_area = (x1_max - x1_min) * (y1_max - y1_min);
        let box2_area = (x2_max - x2_min) * (y2_max - y2_min);
        let union_area = box1_area + box2_area - inter_area;

        if union_area > 0.0 {
            inter_area / union_area
        } else {
            0.0
        }
    }

    /// 获取类别名称列表
    pub fn get_class_names(&self) -> &[String] {
        &self.class_names
    }

    /// 获取输入尺寸
    pub fn get_input_size(&self) -> (u32, u32) {
        (self.input_width, self.input_height)
    }
}
