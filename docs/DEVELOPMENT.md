# 开发指南

本文档面向 Oria 项目的开发者。

## 📋 目录

- [环境搭建](#环境搭建)
- [项目结构](#项目结构)
- [开发流程](#开发流程)
- [技术选型](#技术选型)
- [最佳实践](#最佳实践)
- [调试技巧](#调试技巧)

---

## 环境搭建

### 必需工具

1. **Node.js** (>= 18)
   - 下载：https://nodejs.org/
   - 推荐使用 LTS 版本

2. **pnpm** (>= 8)
   ```bash
   npm install -g pnpm
   ```

3. **Rust** (>= 1.70)
   ```bash
   # Windows (使用 rustup-init.exe)
   https://www.rust-lang.org/tools/install
   
   # Linux/macOS
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

4. **系统依赖**

   **Windows:**
   - Visual Studio Build Tools
   - WebView2 Runtime

   **Linux:**
   ```bash
   sudo apt-get update
   sudo apt-get install -y libgtk-3-dev libwebkit2gtk-4.0-dev \
     libappindicator3-dev librsvg2-dev patchelf
   ```

   **macOS:**
   ```bash
   xcode-select --install
   ```

### 克隆项目

```bash
git clone https://github.com/yourusername/oria.git
cd oria
```

### 安装依赖

```bash
# 安装前端依赖
pnpm install

# 安装 Rust 依赖（自动）
# Cargo 会在首次构建时自动下载
```

### 运行开发服务器

```bash
pnpm tauri dev
```

首次运行需要编译 Rust 代码，可能需要几分钟。

---

## 项目结构

```
oria/
├── src/                          # Vue 前端
│   ├── components/               # UI 组件
│   │   ├── ToolBar.vue          # 工具栏
│   │   ├── FileList.vue         # 文件列表
│   │   ├── AnnotationCanvas.vue # 标注画布
│   │   ├── PropertyPanel.vue    # 属性面板
│   │   ├── AutoLabelDialog.vue  # AI标注对话框
│   │   ├── InferenceSettings.vue# 推理设置
│   │   └── KeyboardHelp.vue     # 快捷键帮助
│   ├── utils/                   # 工具函数
│   │   ├── annotation.js        # 标注相关
│   │   └── keyboard.js          # 键盘事件
│   ├── App.vue                  # 主组件
│   ├── main.js                  # 入口文件
│   └── style.css                # 全局样式
│
├── src-tauri/                   # Tauri 后端
│   ├── src/
│   │   ├── core/               # 核心模块
│   │   │   ├── annotation.rs   # 标注数据
│   │   │   ├── cache.rs        # 缓存管理
│   │   │   └── image.rs        # 图像处理
│   │   ├── inference/          # 推理模块
│   │   │   ├── api_client.rs   # API客户端
│   │   │   ├── model_inference.rs
│   │   │   └── onnx_inference.rs
│   │   ├── export/             # 导出模块
│   │   │   ├── coco.rs         # COCO格式
│   │   │   └── yolo.rs         # YOLO格式
│   │   ├── ui/                 # UI交互
│   │   │   ├── file_dialog.rs  # 文件对话框
│   │   │   └── image_loader.rs # 图像加载
│   │   ├── lib.rs              # 库入口
│   │   ├── main.rs             # 主入口
│   │   ├── models.rs           # 数据模型
│   │   └── error.rs            # 错误处理
│   ├── Cargo.toml              # Rust 依赖
│   └── tauri.conf.json         # Tauri 配置
│
├── python_api_server/           # Python API 服务
│   ├── app.py                  # FastAPI 应用
│   ├── requirements.txt        # Python 依赖
│   └── README.md               # 说明文档
│
├── docs/                        # 文档
└── .github/                     # GitHub 配置
```

---

## 开发流程

### 1. 创建功能分支

```bash
git checkout -b feature/your-feature-name
```

### 2. 开发

#### 前端开发

文件位置：`src/`

热重载：前端代码修改后会自动重载，无需重启。

#### 后端开发

文件位置：`src-tauri/src/`

热重载：Rust 代码修改后需要重新编译，Tauri 会自动重启。

### 3. 测试

```bash
# 运行前端测试
pnpm test

# 运行 Rust 测试
cd src-tauri
cargo test
```

### 4. 代码格式化

```bash
# 前端
pnpm run format

# 后端
cd src-tauri
cargo fmt
```

### 5. 代码检查

```bash
# 前端
pnpm run lint

# 后端
cd src-tauri
cargo clippy
```

### 6. 提交代码

```bash
git add .
git commit -m "feat: add awesome feature"
git push origin feature/your-feature-name
```

### 7. 创建 Pull Request

在 GitHub 上创建 PR，等待审核。

---

## 技术选型

### 前端技术栈

| 技术 | 版本 | 用途 |
|-----|------|-----|
| Vue | 3.5+ | 前端框架 |
| Naive UI | 2.43+ | UI 组件库 |
| Vite | 6.0+ | 构建工具 |
| Canvas API | - | 图形渲染 |

### 后端技术栈

| 技术 | 版本 | 用途 |
|-----|------|-----|
| Tauri | 2.0+ | 桌面框架 |
| Rust | 1.70+ | 系统语言 |
| tokio | 1.x | 异步运行时 |
| reqwest | 0.11+ | HTTP 客户端 |
| image | 0.24+ | 图像处理 |
| serde | 1.x | 序列化 |

### AI 推理

| 技术 | 版本 | 用途 |
|-----|------|-----|
| ONNX Runtime | - | 本地推理 |
| FastAPI | 0.109+ | API 服务 |
| Ultralytics | 8.x | YOLOv8 |

---

## 最佳实践

### Vue 组件开发

#### 使用 Composition API

```vue
<script setup>
import { ref, computed, onMounted } from 'vue';

// 响应式状态
const count = ref(0);

// 计算属性
const doubled = computed(() => count.value * 2);

// 生命周期
onMounted(() => {
  console.log('Component mounted');
});
</script>
```

#### 组件通信

```vue
<!-- 父组件 -->
<ChildComponent 
  :prop-value="value"
  @child-event="handleEvent"
/>

<!-- 子组件 -->
<script setup>
const props = defineProps(['propValue']);
const emit = defineEmits(['child-event']);

const handleClick = () => {
  emit('child-event', data);
};
</script>
```

### Rust 开发

#### Tauri 命令

```rust
#[tauri::command]
async fn my_command(param: String) -> Result<String, String> {
    // 处理逻辑
    Ok("success".to_string())
}
```

#### 错误处理

```rust
use anyhow::{Context, Result};

fn process_data() -> Result<Data> {
    let data = read_file()
        .context("Failed to read file")?;
    Ok(data)
}
```

#### 异步操作

```rust
use tokio::fs;

#[tauri::command]
async fn async_operation() -> Result<String, String> {
    let content = fs::read_to_string("file.txt")
        .await
        .map_err(|e| e.to_string())?;
    Ok(content)
}
```

### 性能优化

1. **图片缓存**
   - 使用 LRU 缓存策略
   - 预加载相邻图片

2. **Canvas 渲染**
   - 使用 requestAnimationFrame
   - 避免频繁重绘

3. **数据结构**
   - 使用 HashMap 加速查找
   - 使用 Vec 存储列表

---

## 调试技巧

### 前端调试

#### 开发者工具

```bash
# 在开发模式下按 F12 打开
pnpm tauri dev
```

#### Vue Devtools

安装 Vue Devtools 浏览器扩展。

#### 日志输出

```javascript
console.log('Debug info:', data);
console.error('Error:', error);
```

### 后端调试

#### 日志输出

```rust
println!("Debug: {:?}", data);
eprintln!("Error: {}", error);
```

#### 环境变量

```bash
# 启用详细日志
RUST_LOG=debug pnpm tauri dev

# 启用 backtrace
RUST_BACKTRACE=1 pnpm tauri dev
```

#### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function() {
        assert_eq!(add(2, 2), 4);
    }
}
```

---

## 常见问题

### Q: 编译失败怎么办？

A: 尝试以下步骤：
1. 清除缓存：`pnpm clean` 和 `cargo clean`
2. 重新安装依赖
3. 检查 Rust 版本

### Q: 热重载不工作？

A: 
- 前端：检查 Vite 服务器是否运行
- 后端：Rust 代码需要重新编译

### Q: 如何添加新的 Tauri 命令？

A:
1. 在 `src-tauri/src/lib.rs` 中定义命令
2. 在 `invoke_handler!` 中注册
3. 在前端使用 `invoke('command_name')`

---

## 资源链接

- [Tauri 文档](https://tauri.app/zh-cn/)
- [Vue 3 文档](https://cn.vuejs.org/)
- [Naive UI 文档](https://www.naiveui.com/zh-CN/)
- [Rust 文档](https://doc.rust-lang.org/book/)

---

*最后更新：2025-01*

