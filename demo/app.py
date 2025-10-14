"""
YOLOv8 推理 API 服务端 - Demo

这是一个简单的FastAPI服务，用于提供YOLOv8模型推理API。
auto-label项目会调用这个API来进行自动标注。

使用方法:
1. 安装依赖: pip install fastapi uvicorn ultralytics pillow numpy
2. 运行服务: python app.py
3. 服务将在 http://localhost:8000 启动

API端点:
- GET  /health      - 健康检查
- GET  /model_info  - 获取模型信息
- POST /predict     - 推理接口
"""

from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel
from typing import List
import base64
import io
import time
from PIL import Image
import numpy as np

# 初始化FastAPI应用
app = FastAPI(
    title="YOLOv8 推理服务",
    description="为auto-label项目提供模型推理API",
    version="1.0.0"
)

# 配置CORS，允许Tauri应用调用
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],  # 生产环境应该限制具体域名
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# 全局变量存储模型
model = None
model_info = {
    "model_name": "YOLOv8n",
    "class_names": [],
    "input_size": [640, 640]
}


def load_model(model_path: str = "yolov8n.pt"):
    """加载YOLOv8模型"""
    global model, model_info
    try:
        from ultralytics import YOLO
        model = YOLO(model_path)
        
        # 获取类别名称
        if hasattr(model, 'names'):
            model_info["class_names"] = list(model.names.values())
        else:
            # 默认COCO类别
            model_info["class_names"] = [
                'person', 'bicycle', 'car', 'motorcycle', 'airplane', 'bus', 'train', 'truck', 'boat',
                'traffic light', 'fire hydrant', 'stop sign', 'parking meter', 'bench', 'bird', 'cat',
                'dog', 'horse', 'sheep', 'cow', 'elephant', 'bear', 'zebra', 'giraffe', 'backpack',
                'umbrella', 'handbag', 'tie', 'suitcase', 'frisbee', 'skis', 'snowboard', 'sports ball',
                'kite', 'baseball bat', 'baseball glove', 'skateboard', 'surfboard', 'tennis racket',
                'bottle', 'wine glass', 'cup', 'fork', 'knife', 'spoon', 'bowl', 'banana', 'apple',
                'sandwich', 'orange', 'broccoli', 'carrot', 'hot dog', 'pizza', 'donut', 'cake', 'chair',
                'couch', 'potted plant', 'bed', 'dining table', 'toilet', 'tv', 'laptop', 'mouse', 'remote',
                'keyboard', 'cell phone', 'microwave', 'oven', 'toaster', 'sink', 'refrigerator', 'book',
                'clock', 'vase', 'scissors', 'teddy bear', 'hair drier', 'toothbrush'
            ]
        
        print(f"✓ 模型加载成功: {model_path}")
        print(f"✓ 类别数量: {len(model_info['class_names'])}")
        return True
    except ImportError:
        print("✗ 错误: 未安装ultralytics库")
        print("  请运行: pip install ultralytics")
        return False
    except Exception as e:
        print(f"✗ 模型加载失败: {e}")
        return False


# 请求和响应模型
class InferenceRequest(BaseModel):
    image_base64: str
    conf_threshold: float = 0.25
    iou_threshold: float = 0.45


class Detection(BaseModel):
    class_id: int
    class_name: str
    confidence: float
    bbox: List[float]  # [x_min, y_min, x_max, y_max]


class InferenceResponse(BaseModel):
    detections: List[Detection]
    inference_time_ms: float


class ModelInfo(BaseModel):
    model_name: str
    class_names: List[str]
    input_size: List[int]


# API端点
@app.get("/")
async def root():
    """根路径"""
    return {
        "service": "YOLOv8 推理服务",
        "status": "running",
        "model_loaded": model is not None,
        "endpoints": {
            "health": "/health",
            "model_info": "/model_info",
            "predict": "/predict (POST)"
        }
    }


@app.get("/health")
async def health_check():
    if model is None:
        raise HTTPException(status_code=503, detail="模型未加载")
    return {"status": "healthy", "model": model_info["model_name"]}


@app.get("/model_info", response_model=ModelInfo)
async def get_model_info():
    """获取模型信息"""
    if model is None:
        raise HTTPException(status_code=503, detail="模型未加载")
    return model_info


@app.post("/predict", response_model=InferenceResponse)
async def predict(request: InferenceRequest):
    """推理接口"""
    if model is None:
        raise HTTPException(status_code=503, detail="模型未加载")
    
    try:
        # 解码base64图片
        image_data = base64.b64decode(request.image_base64)
        image = Image.open(io.BytesIO(image_data))
        
        # 转换为RGB (如果是RGBA或其他格式)
        if image.mode != 'RGB':
            image = image.convert('RGB')
        
        # 推理
        start_time = time.time()
        results = model(
            image,
            conf=request.conf_threshold,
            iou=request.iou_threshold,
            verbose=False
        )
        inference_time = (time.time() - start_time) * 1000  # 转换为毫秒
        
        # 解析结果
        detections = []
        if len(results) > 0 and results[0].boxes is not None:
            boxes = results[0].boxes
            for i in range(len(boxes)):
                # 获取边界框坐标 (xyxy格式)
                bbox = boxes.xyxy[i].cpu().numpy().tolist()
                
                # 获取类别ID和置信度
                class_id = int(boxes.cls[i].cpu().numpy())
                confidence = float(boxes.conf[i].cpu().numpy())
                
                # 获取类别名称
                class_name = model_info["class_names"][class_id] if class_id < len(model_info["class_names"]) else f"class_{class_id}"
                
                detections.append(Detection(
                    class_id=class_id,
                    class_name=class_name,
                    confidence=confidence,
                    bbox=bbox
                ))
        
        return InferenceResponse(
            detections=detections,
            inference_time_ms=inference_time
        )
    
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"推理失败: {str(e)}")


# 启动配置
if __name__ == "__main__":
    import uvicorn
    import sys
    
    print("=" * 60)
    print("YOLOv8 推理API服务")
    print("=" * 60)
    
    # 检查命令行参数
    model_path = r"E:\yan\wd\大论文\ultralytics\runs\train\v8s\weights\best.pt"
    if len(sys.argv) > 1:
        model_path = sys.argv[1]
    
    print(f"\n正在加载模型: {model_path}")
    
    # 加载模型
    if not load_model(model_path):
        print("\n启动失败: 无法加载模型")
        print("\n提示:")
        print("1. 确保已安装ultralytics: pip install ultralytics")
        print("2. 如果是首次运行，yolov8n.pt会自动下载")
        print("3. 也可以指定自己的模型: python app.py /path/to/your/model.pt")
        sys.exit(1)
    
    print("\n" + "=" * 60)
    print("服务启动中...")
    print("API地址: http://localhost:8000")
    print("API文档: http://localhost:8000/docs")
    print("=" * 60 + "\n")
    
    # 启动服务
    uvicorn.run(
        app,
        host="0.0.0.0",
        port=8000,
        log_level="info"
    )




