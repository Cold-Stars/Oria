<div align="center">
  <img src="src-tauri/icons/oria_logo.svg" alt="Oria Logo" width="200"/>
  
  # Oria
  
  **智能目标检测标注工具**
  
  *Orientation + AI —— 为AI指引方向*
  
  [![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
  [![Tauri](https://img.shields.io/badge/Tauri-2.0-orange)](https://tauri.app)
  [![Vue](https://img.shields.io/badge/Vue-3.5-green)](https://vuejs.org)
  [![Rust](https://img.shields.io/badge/Rust-1.70+-red)](https://www.rust-lang.org)
  
  [English](README_EN.md) | 简体中文
  
</div>

---

## ✨ 项目简介

**Oria** 是一款现代化的目标检测标注工具，致力于让数据标注工作更高效、更智能。

### 🎯 名字来源

**Oria** = **Ori**entation + **A**I

- **Orientation**（定向）：代表标注框为物体提供精确的空间定位
- **AI**（人工智能）：强调智能化的自动标注能力
- **寓意**：为AI的学习指引正确的方向

### 🌟 核心特性

- 🤖 **AI 辅助标注** - 支持 YOLOv8 模型，API/ONNX 双模式推理
- 🎨 **现代化界面** - 基于 Naive UI 的简洁美观设计
- ⚡ **高性能渲染** - Canvas 引擎，流畅处理大量标注
- 🔄 **多格式支持** - COCO、YOLOv8 格式导入导出
- 🎯 **双标注模式** - 支持矩形框和旋转框标注
- ⌨️ **快捷键操作** - 完善的键盘快捷键支持
- 🚀 **跨平台** - Windows、macOS、Linux 全平台支持
- 💾 **智能缓存** - 图片预加载与智能缓存机制

---

## 📸 界面预览

<div align="center">
  
  ### 主界面
  *标注工作区、文件列表、属性面板*
  
  ![主界面截图](docs/images/main-interface.png)
  
  ### AI 自动标注
  *一键智能标注，大幅提升工作效率*
  
  ![AI标注截图](docs/images/ai-labeling.png)
  
</div>

---

## 🚀 快速开始

### 📋 环境要求

- **Node.js** >= 18
- **Rust** >= 1.70
- **pnpm** >= 8.0

### 📦 安装

```bash
# 克隆项目
git clone https://github.com/yourusername/oria.git
cd oria

# 安装依赖
pnpm install
```

### 🔧 开发

```bash
# 启动开发服务器
pnpm tauri dev
```

### 📦 构建

```bash
# 构建生产版本
pnpm tauri build
```

构建完成后，安装包位于 `src-tauri/target/release/bundle/` 目录。

---

## 🛠️ 技术栈

### 前端

- **框架**: Vue 3 (Composition API)
- **UI 库**: Naive UI
- **构建工具**: Vite 6
- **状态管理**: Vue Reactivity
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

---

## 📚 功能详解

### 🎯 标注功能

#### 矩形框标注
- 支持标准水平矩形框
- 拖拽创建，精确控制
- 实时预览，所见即所得

#### 旋转框标注
- 支持任意角度旋转
- 适用于斜向目标
- OBB (Oriented Bounding Box) 格式

### 🤖 AI 辅助标注

#### API 模式
```bash
# 启动 Python 推理服务
cd python_api_server
pip install -r requirements.txt
python app.py
```

#### ONNX 模式
- 无需网络连接
- GPU 加速支持
- 模型本地化部署

详细配置参考：[AI 标注指南](docs/AI_LABELING_GUIDE.md)

### 📤 数据导出

#### COCO 格式
```json
{
  "images": [...],
  "annotations": [...],
  "categories": [...]
}
```

#### YOLOv8 格式
```
class_id x_center y_center width height
```

---

## ⌨️ 快捷键

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
- `Delete/Backspace` - 删除选中
- `Escape` - 取消选择
- `Ctrl+D` - 复制标注

---

## 📁 项目结构

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
│
├── docs/                        # 文档
│   ├── AI_LABELING_GUIDE.md    # AI标注指南
│   └── DEVELOPMENT.md          # 开发指南
│
└── README.md                    # 项目说明
```

---

## 🤝 参与贡献

我们欢迎所有形式的贡献！

### 贡献方式

1. 🐛 报告 Bug
2. 💡 提出新功能建议
3. 📝 改进文档
4. 🔧 提交代码

### 开发流程

```bash
# 1. Fork 项目
# 2. 创建特性分支
git checkout -b feature/amazing-feature

# 3. 提交更改
git commit -m 'feat: add amazing feature'

# 4. 推送到分支
git push origin feature/amazing-feature

# 5. 提交 Pull Request
```

### 代码规范

- **前端**: ESLint + Prettier
- **后端**: rustfmt + clippy
- **提交**: 遵循 [Conventional Commits](https://www.conventionalcommits.org/)

详细开发指南参考：[开发文档](docs/DEVELOPMENT.md)

---

## 🗺️ 路线图

### ✅ 已完成

- [x] 基础标注功能（矩形框、旋转框）
- [x] COCO/YOLO 格式支持
- [x] AI 自动标注（API + ONNX）
- [x] 图片缓存与预加载
- [x] 快捷键系统
- [x] 深色模式支持

### 🚧 进行中

- [ ] 撤销/重做功能
- [ ] 批量标注优化
- [ ] 性能优化

### 🔮 计划中

- [ ] 语义分割标注
- [ ] 关键点标注
- [ ] 多边形标注
- [ ] 数据增强预览
- [ ] 标注质量检查
- [ ] 团队协作功能
- [ ] 云端同步
- [ ] 插件系统

---

## 📊 统计

<div align="center">
  
  ![Star History Chart](https://api.star-history.com/svg?repos=yourusername/oria&type=Date)
  
</div>

---

## 📄 许可证

本项目采用 [MIT License](LICENSE) 开源协议。

```
MIT License

Copyright (c) 2025 Oria Contributors

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.
```

---

## 🙏 致谢

感谢以下开源项目：

- [Tauri](https://tauri.app/) - 跨平台桌面应用框架
- [Vue.js](https://vuejs.org/) - 渐进式 JavaScript 框架
- [Naive UI](https://www.naiveui.com/) - Vue 3 组件库
- [Ultralytics](https://github.com/ultralytics/ultralytics) - YOLOv8 实现
- [ONNX Runtime](https://onnxruntime.ai/) - 高性能推理引擎

---

## 💬 社区

- 💬 [Discussions](../../discussions) - 讨论和问答
- 🐛 [Issues](../../issues) - Bug 报告和功能请求
- 📧 Email: your-email@example.com

---

## ⭐ Star History

如果这个项目对你有帮助，请给我们一个 Star ⭐️

<div align="center">
  
  [![Star History Chart](https://api.star-history.com/svg?repos=yourusername/oria&type=Timeline)](https://star-history.com/#yourusername/oria&Timeline)
  
</div>

---

<div align="center">
  
  **[官网](https://oria-label.com)** • **[文档](https://docs.oria-label.com)** • **[更新日志](CHANGELOG.md)**
  
  用 ❤️ 打造
  
</div>
