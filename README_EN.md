<div align="center">
  <img src="src-tauri/icons/oria_logo.svg" alt="Oria Logo" width="200"/>
  
  # Oria
  
  **Intelligent Object Detection Annotation Tool**
  
  *Orientation + AI — Guiding the Direction for AI*
  
  [![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
  [![Tauri](https://img.shields.io/badge/Tauri-2.0-orange)](https://tauri.app)
  [![Vue](https://img.shields.io/badge/Vue-3.5-green)](https://vuejs.org)
  [![Rust](https://img.shields.io/badge/Rust-1.70+-red)](https://www.rust-lang.org)
  
  English | [简体中文](README.md)
  
</div>

---

## ✨ Introduction

**Oria** is a modern object detection annotation tool designed to make data labeling more efficient and intelligent.

### 🎯 Name Origin

**Oria** = **Ori**entation + **A**I

- **Orientation**: Represents precise spatial positioning for objects through bounding boxes
- **AI**: Emphasizes intelligent auto-annotation capabilities
- **Meaning**: Guiding AI learning in the right direction

### 🌟 Key Features

- 🤖 **AI-Assisted Annotation** - YOLOv8 support with API/ONNX dual-mode inference
- 🎨 **Modern UI** - Clean and beautiful design based on Naive UI
- ⚡ **High Performance** - Canvas engine for smooth handling of large annotations
- 🔄 **Multi-Format Support** - COCO and YOLOv8 format import/export
- 🎯 **Dual Annotation Modes** - Rectangle and rotated bounding box support
- ⌨️ **Keyboard Shortcuts** - Comprehensive keyboard shortcut support
- 🚀 **Cross-Platform** - Windows, macOS, and Linux support
- 💾 **Smart Caching** - Image preloading and intelligent cache mechanism

---

## 🚀 Quick Start

### 📋 Requirements

- **Node.js** >= 18
- **Rust** >= 1.70
- **pnpm** >= 8.0

### 📦 Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/oria.git
cd oria

# Install dependencies
pnpm install
```

### 🔧 Development

```bash
# Start development server
pnpm tauri dev
```

### 📦 Build

```bash
# Build for production
pnpm tauri build
```

After building, installers will be located in `src-tauri/target/release/bundle/`.

---

## 🛠️ Tech Stack

### Frontend

- **Framework**: Vue 3 (Composition API)
- **UI Library**: Naive UI
- **Build Tool**: Vite 6
- **State Management**: Vue Reactivity
- **Graphics**: Canvas API

### Backend

- **Desktop Framework**: Tauri 2.0
- **Language**: Rust
- **Image Processing**: image crate
- **Async Runtime**: tokio
- **HTTP Client**: reqwest

### AI Inference

- **Local Inference**: ONNX Runtime
- **Remote Inference**: FastAPI + YOLOv8
- **Model Support**: YOLOv8 series

---

## ⌨️ Keyboard Shortcuts

### File Operations
- `Ctrl+O` - Open folder
- `Ctrl+S` - Save annotations
- `Ctrl+E` - Export annotations

### Tool Switching
- `S` - Select tool
- `R` - Rectangle annotation
- `O` - Rotated rectangle

### Navigation
- `←/→` - Previous/Next image
- `Home/End` - First/Last image
- `Ctrl+F` - Search files

### Annotation Operations
- `Delete/Backspace` - Delete selected
- `Escape` - Cancel selection
- `Ctrl+D` - Duplicate annotation

---

## 🤝 Contributing

We welcome all forms of contributions!

Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details on our code of conduct and the process for submitting pull requests.

---

## 🗺️ Roadmap

### ✅ Completed

- [x] Basic annotation features (rectangles, rotated boxes)
- [x] COCO/YOLO format support
- [x] AI auto-annotation (API + ONNX)
- [x] Image caching and preloading
- [x] Keyboard shortcuts system
- [x] Dark mode support

### 🚧 In Progress

- [ ] Undo/Redo functionality
- [ ] Batch annotation optimization
- [ ] Performance optimization

### 🔮 Planned

- [ ] Semantic segmentation annotation
- [ ] Keypoint annotation
- [ ] Polygon annotation
- [ ] Data augmentation preview
- [ ] Annotation quality check
- [ ] Team collaboration features
- [ ] Cloud synchronization
- [ ] Plugin system

---

## 📄 License

This project is licensed under the [MIT License](LICENSE).

---

## 🙏 Acknowledgements

Thanks to the following open source projects:

- [Tauri](https://tauri.app/) - Cross-platform desktop application framework
- [Vue.js](https://vuejs.org/) - Progressive JavaScript framework
- [Naive UI](https://www.naiveui.com/) - Vue 3 component library
- [Ultralytics](https://github.com/ultralytics/ultralytics) - YOLOv8 implementation
- [ONNX Runtime](https://onnxruntime.ai/) - High-performance inference engine

---

## 💬 Community

- 💬 [Discussions](../../discussions) - Q&A and discussions
- 🐛 [Issues](../../issues) - Bug reports and feature requests
- 📧 Email: your-email@example.com

---

## ⭐ Star History

If this project helps you, please give us a Star ⭐️

<div align="center">
  
  [![Star History Chart](https://api.star-history.com/svg?repos=yourusername/oria&type=Timeline)](https://star-history.com/#yourusername/oria&Timeline)
  
</div>

---

<div align="center">
  
  **[Website](https://oria-label.com)** • **[Docs](https://docs.oria-label.com)** • **[Changelog](CHANGELOG.md)**
  
  Made with ❤️
  
</div>

