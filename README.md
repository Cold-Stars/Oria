<div align="center">
  <img src="src-tauri/icons/oria_logo.svg" alt="Oria Logo" width="200"/>
</div>

目标检测标注工具

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Tauri](https://img.shields.io/badge/Tauri-2.0-orange)](https://tauri.app)
[![Vue](https://img.shields.io/badge/Vue-3.5-green)](https://vuejs.org)
[![Rust](https://img.shields.io/badge/Rust-1.70+-red)](https://www.rust-lang.org)

## 简介

Oria 是一个基于 Tauri + Vue 3 的目标检测标注工具，支持矩形框和旋转框标注

## 主要功能

- 矩形框和旋转框标注
- COCO、YOLOv8 格式导入导出
- YOLOv8 模型推理（API/ONNX）
- 图片缓存和预加载
- 完整的快捷键支持
- 撤销/重做
- 跨平台支持

## 快速开始

### 环境要求

- **Node.js** >= 18
- **Rust** >= 1.70
- **pnpm** >= 8.0

### 安装

```bash
# 克隆项目
git clone https://github.com/yourusername/oria.git
cd oria

# 安装依赖
pnpm install
```

### 开发

```bash
# 启动开发服务器
pnpm tauri dev
```

### 构建

```bash
# 构建生产版本
pnpm tauri build
```

构建完成后，安装包位于 `src-tauri/target/release/bundle/` 目录。

## 技术栈

### 前端

- **框架**: Vue 3 (Composition API)
- **UI 库**: Naive UI
- **构建工具**: Vite 6
- **图形渲染**: Canvas API

### 后端

- **桌面框架**: Tauri 2.0
- **语言**: Rust
- **图像处理**: image crate
- **异步运行时**: tokio
- **HTTP 客户端**: reqwest

### AI 推理

- **本地推理**: ONNX Runtime
- **远程推理**: FastAPI + YOLOv8
- **模型支持**: YOLOv8 系列

## AI 推理配置

### API 模式

```bash
# 启动 Python 推理服务
cd python_api_server
pip install -r requirements.txt
python app.py
```

### ONNX 模式

需要编译时启用 `onnx` feature：

```bash
pnpm tauri dev --features onnx
```

## 数据格式

### COCO 格式

```json
{
  "images": [...],
  "annotations": [...],
  "categories": [...]
}
```

### YOLOv8 格式

```
class_id x_center y_center width height
```

## 快捷键

### 文件操作

- `Ctrl+O` - 打开文件夹
- `Ctrl+S` - 保存标注
- `Ctrl+E` - 导出标注

### 工具切换

- `S` - 选择工具
- `R` - 矩形标注
- `O` - 旋转矩形

### 导航

- `←/→` - 上一张/下一张
- `Home/End` - 首张/末张
- `Ctrl+F` - 搜索文件

### 标注操作

- `Delete` - 删除选中
- `Escape` - 取消选择
- `Ctrl+D` - 复制标注
- `Ctrl+Z` - 撤销
- `Ctrl+Y` - 重做

### 视图操作

- `Ctrl+0` - 重置缩放
- `Ctrl+1` - 适应窗口
- `Ctrl+=` - 放大
- `Ctrl+-` - 缩小
- `H` - 显示/隐藏标注

## 项目结构

```
oria/
├── src/                          # Vue 前端源码
│   ├── components/               # 组件
│   │   ├── ToolBar.vue          # 工具栏
│   │   ├── FileList.vue         # 文件列表
│   │   ├── AnnotationCanvas.vue # 标注画布
│   │   ├── PropertyPanel.vue    # 属性面板
│   │   └── AutoLabelDialog.vue  # AI标注对话框
│   ├── utils/                   # 工具函数
│   ├── App.vue                  # 主应用
│   └── main.js                  # 入口文件
│
├── src-tauri/                   # Tauri 后端
│   ├── src/
│   │   ├── core/               # 核心模块
│   │   ├── inference/          # 推理模块
│   │   ├── export/             # 导出模块
│   │   └── ui/                 # UI交互
│   ├── icons/                  # 应用图标
│   │   ├── oria_icon.svg       # 应用图标
│   │   └── oria_logo.svg       # 项目 Logo
│   └── Cargo.toml              # Rust 依赖
│
├── python_api_server/           # Python 推理服务
│   ├── app.py                  # FastAPI 服务
│   ├── requirements.txt        # Python 依赖
│   └── README.md               # 服务说明
└── README.md                    # 项目说明
```

## 许可证

MIT License
