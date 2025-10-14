# 删除预览图缓存功能

## 修改原因

在修复大图标注位置问题后，前端已改为直接使用原图尺寸作为画布逻辑坐标，不再使用预览图。因此预览图缓存功能已废弃，应该删除以简化代码。

## 删除的功能

### 1. **预览图生成和缓存**
- 之前：大图(>1920px)会生成1024px的预览图用于快速显示
- 现在：所有图片都直接加载完整图

### 2. **预加载策略**
- 之前：大文件(>2MB)预加载预览图，小文件预加载完整图
- 现在：统一预加载完整图

## 修改的文件

### 1. **src-tauri/src/ui/image_loader.rs**

#### load_image_cached 函数
**修改前**（23-57行）：
```rust
// 根据图片大小选择加载策略
let (url, display_width, display_height) = if width > 1920 || height > 1920 {
    match IMAGE_CACHE.get_preview(&path) {
        Ok(url) => (url, Some(preview_w), Some(preview_h)),
        Err(_) => {
            let url = IMAGE_CACHE.get_full_image(&path)?;
            (url, None, None)
        }
    }
} else {
    let url = IMAGE_CACHE.get_full_image(&path)?;
    (url, None, None)
};

Ok(ImageData {
    url,
    width,
    height,
    file_path: path,
    display_width,
    display_height,
})
```

**修改后**（23-31行）：
```rust
// 直接加载完整图
let url = IMAGE_CACHE.get_full_image(&path)?;

Ok(ImageData {
    url,
    width,
    height,
    file_path: path,
})
```

#### preload_images 函数
**修改前**（138-151行）：
```rust
for path in paths_to_preload {
    if let Ok(metadata) = tokio::fs::metadata(&path).await {
        let file_size = metadata.len();
        // 大文件（>2MB）只预加载预览图
        if file_size > 2_000_000 {
            let _ = IMAGE_CACHE.get_preview(&path);
        } else {
            // 小文件直接预加载完整图
            let _ = IMAGE_CACHE.get_full_image(&path);
        }
    }
}
```

**修改后**（111-113行）：
```rust
for path in paths_to_preload {
    let _ = IMAGE_CACHE.get_full_image(&path);
}
```

#### get_cache_stats 函数
删除了 `preview_cache_count` 和 `preview_cache_max` 字段。

### 2. **src-tauri/src/models.rs**

**修改前**（3-14行）：
```rust
pub struct ImageData {
    pub url: String,
    pub width: u32,
    pub height: u32,
    pub file_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_width: Option<u32>,   // 删除
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_height: Option<u32>,  // 删除
}
```

**修改后**（3-10行）：
```rust
pub struct ImageData {
    pub url: String,
    pub width: u32,
    pub height: u32,
    pub file_path: String,
}
```

### 3. **src-tauri/src/core/cache.rs**

#### 删除的内容
1. **ImageCache 结构**
   - 删除 `preview_cache` 字段
   - 删除 `max_preview_cache` 字段

2. **构造函数**
   - 从 `new(max_full_cache, max_thumbnail_cache, max_preview_cache)` 
   - 改为 `new(max_full_cache, max_thumbnail_cache)`

3. **方法**
   - 删除 `get_preview()` 方法（90-109行）
   - 删除 `cache_preview()` 方法（225-250行）

4. **clear() 方法**
   - 删除 `self.preview_cache.lock().unwrap().clear();`

5. **stats() 方法和 CacheStats 结构**
   - 删除 `preview_cache_count` 字段
   - 删除 `preview_cache_max` 字段

6. **全局实例**
   - 从 `ImageCache::new(10, 200, 30)` 
   - 改为 `ImageCache::new(10, 200)`

## 保留的功能

### ✅ 文件列表缩略图（64px）
- **位置**: `src/components/FileList.vue`
- **用途**: 在左侧文件列表显示小缩略图
- **状态**: 保留，继续使用

### ✅ 完整图片缓存
- **位置**: `src-tauri/src/core/cache.rs`
- **用途**: 缓存最近加载的完整图片
- **缓存数量**: 10张

### ✅ 缩略图缓存
- **位置**: `src-tauri/src/core/cache.rs`
- **用途**: 缓存文件列表的小缩略图
- **缓存数量**: 200张

## 缓存架构变化

### 修改前：三级缓存
```
完整图缓存 (10张)
    ↓
预览图缓存 (30张, 1024px)  ← 已删除
    ↓
缩略图缓存 (200张, 64px)
```

### 修改后：二级缓存
```
完整图缓存 (10张)
    ↓
缩略图缓存 (200张, 64px)
```

## 性能影响

### 优点
- ✅ 代码更简洁，维护更容易
- ✅ 减少了缓存管理的复杂度
- ✅ 标注位置完全准确（基于原图尺寸）

### 实际表现
- ✅ 实测发现加载原图比预览图更快
- ✅ 没有性能问题

## 向后兼容

- ✅ 标注文件格式未改变
- ✅ 前端显示逻辑已适配
- ✅ 所有功能正常工作

## 代码统计

| 文件 | 删除行数 | 简化内容 |
|------|---------|---------|
| image_loader.rs | ~35行 | 加载逻辑、预加载逻辑 |
| models.rs | ~4行 | ImageData 字段 |
| cache.rs | ~70行 | 预览图缓存完整功能 |
| **总计** | **~110行** | **大幅简化** |

## 总结

删除预览图功能后：
1. 代码更简洁清晰
2. 缓存逻辑更简单
3. 标注位置更准确
4. 性能没有下降
5. 维护成本降低

这是一次成功的代码重构和简化！🎉





