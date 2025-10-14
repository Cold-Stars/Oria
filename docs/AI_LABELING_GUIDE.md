# AI 自动标注使用指南

本指南将详细介绍如何在 Oria 中使用 AI 自动标注功能。

## 📋 目录

- [功能概述](#功能概述)
- [API 推理模式](#api-推理模式)
- [ONNX 推理模式](#onnx-推理模式)
- [推理参数配置](#推理参数配置)
- [常见问题](#常见问题)

---

## 功能概述

Oria 支持两种 AI 推理方式：

1. **API 推理**：通过 HTTP API 调用远程推理服务
2. **ONNX 推理**：使用本地 ONNX 模型进行推理

### 支持的模型

目前支持 **YOLOv8** 系列模型：

- YOLOv8n (nano)
- YOLOv8s (small)
- YOLOv8m (medium)
- YOLOv8l (large)
- YOLOv8x (extra large)

---

## API 推理模式

### 1. 启动推理服务

#### 方法一：使用提供的脚本

**Windows:**
```bash
cd python_api_server
start_server.bat
```

**Linux/macOS:**
```bash
cd python_api_server
chmod +x start_server.sh
./start_server.sh
```

#### 方法二：手动启动

```bash
cd python_api_server

# 安装依赖
pip install -r requirements.txt

# 启动服务
python app.py
```

### 2. 配置模型路径

编辑 `python_api_server/app.py` 第 213 行：

```python
model_path = r"path/to/your/model.pt"
```

或者通过命令行参数指定：

```bash
python app.py /path/to/your/model.pt
```

### 3. 在 Oria 中使用

1. 打开 Oria 应用
2. 点击工具栏的 "⚙️ 推理设置" 按钮
3. 选择 "API 推理" 模式
4. 配置 API 地址（默认：`http://localhost:8000`）
5. 点击 "测试连接" 确认服务可用
6. 点击工具栏的推理按钮开始标注

### API 接口说明

#### 健康检查
```
GET /health
```

响应：
```json
{
  "status": "healthy",
  "model": "YOLOv8n"
}
```

#### 获取模型信息
```
GET /model_info
```

响应：
```json
{
  "model_name": "YOLOv8n",
  "class_names": ["person", "bicycle", ...],
  "input_size": [640, 640]
}
```

#### 推理接口
```
POST /predict
Content-Type: application/json

{
  "image_base64": "...",
  "conf_threshold": 0.25,
  "iou_threshold": 0.45
}
```

响应：
```json
{
  "detections": [
    {
      "class_id": 0,
      "class_name": "person",
      "confidence": 0.92,
      "bbox": [100, 150, 300, 400]
    }
  ],
  "inference_time_ms": 45.2
}
```

---

## ONNX 推理模式

### 1. 准备 ONNX 模型

#### 方法一：导出 PyTorch 模型

```python
from ultralytics import YOLO

# 加载 PyTorch 模型
model = YOLO('yolov8n.pt')

# 导出为 ONNX 格式
model.export(format='onnx')
```

#### 方法二：下载预训练模型

访问 [Ultralytics 官网](https://docs.ultralytics.com/) 下载预训练的 ONNX 模型。

### 2. 在 Oria 中使用

1. 打开 Oria 应用
2. 点击工具栏的 "⚙️ 推理设置" 按钮
3. 选择 "ONNX 本地推理" 模式
4. 点击 "选择模型" 按钮，选择 `.onnx` 文件
5. 点击 "验证模型" 确认模型可用
6. （可选）启用 GPU 加速
7. 点击工具栏的推理按钮开始标注

### GPU 加速

#### 前置要求

- NVIDIA GPU（支持 CUDA）
- 安装 CUDA Toolkit
- 安装 cuDNN

#### 启用 GPU

在推理设置中，勾选 "使用 GPU 加速" 选项。

> ⚠️ **注意**：首次使用 GPU 可能需要额外下载 CUDA 相关的库文件。

---

## 推理参数配置

### 置信度阈值 (Confidence Threshold)

- **范围**：0.1 - 0.9
- **默认值**：0.25
- **说明**：只有置信度高于此阈值的检测结果会被保留
- **建议**：
  - 提高阈值：减少误检，但可能漏检
  - 降低阈值：增加检出率，但可能增加误检

### IOU 阈值 (IoU Threshold)

- **范围**：0.1 - 0.9
- **默认值**：0.45
- **说明**：用于 NMS（非极大值抑制）去除重叠的检测框
- **建议**：
  - 提高阈值：保留更多重叠的框
  - 降低阈值：更激进地去除重叠框

### 推理数量

- **当前图**：只推理当前选中的图片
- **5张**：推理当前开始的5张图片
- **全部**：推理所有剩余图片
- **自定义**：指定推理数量

---

## 工作流程建议

### 方案一：API + 快速迭代

适合场景：
- 有多台设备需要标注
- 需要频繁更换模型
- 团队协作

步骤：
1. 在服务器上部署 API 服务
2. 所有客户端连接到同一服务
3. 统一管理模型版本

### 方案二：ONNX + 本地化

适合场景：
- 无网络环境
- 需要高性能
- 数据隐私要求高

步骤：
1. 导出并分发 ONNX 模型
2. 每个客户端本地推理
3. 启用 GPU 加速提升性能

### 混合方案

1. 初次标注使用 API 快速完成
2. 精细调整使用手动标注
3. 特殊情况使用 ONNX 本地推理

---

## 常见问题

### Q: API 连接失败怎么办？

A: 检查以下几点：
1. 确认 Python 服务已启动
2. 检查端口是否被占用（默认 8000）
3. 检查防火墙设置
4. 确认 API 地址配置正确

### Q: ONNX 模型加载失败？

A: 可能的原因：
1. 模型文件损坏
2. 模型格式不兼容
3. 缺少必要的依赖库

解决方法：
1. 重新导出模型
2. 确认使用的是 YOLOv8 模型
3. 检查 ONNX Runtime 版本

### Q: GPU 加速不生效？

A: 检查：
1. 是否安装了 CUDA 和 cuDNN
2. ONNX Runtime GPU 版本是否正确
3. 查看系统日志了解详细错误

### Q: 推理速度很慢？

A: 优化建议：
1. 使用 GPU 加速
2. 选择更小的模型（如 YOLOv8n）
3. 降低图片分辨率
4. 使用 ONNX 本地推理

### Q: 标注结果不准确？

A: 改进方法：
1. 调整置信度阈值
2. 使用更大的模型
3. 使用在相似数据上训练的模型
4. 手动调整和修正结果

---

## 性能参考

### API 推理（YOLOv8n，NVIDIA RTX 3060）

| 图片尺寸 | 推理时间 |
|---------|---------|
| 640x640 | ~20ms   |
| 1280x1280 | ~45ms |
| 1920x1080 | ~60ms |

### ONNX 推理

| 模型 | CPU (i7-12700K) | GPU (RTX 3060) |
|------|----------------|----------------|
| YOLOv8n | ~150ms | ~15ms |
| YOLOv8s | ~280ms | ~25ms |
| YOLOv8m | ~520ms | ~40ms |

---

## 技术支持

如有问题，请通过以下方式联系：

- 📝 [提交 Issue](../../issues)
- 💬 [参与讨论](../../discussions)
- 📧 Email: your-email@example.com

---

*最后更新：2025-01*

