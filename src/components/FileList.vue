<script setup>
import { ref, computed, watch, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

const emit = defineEmits(["select-file"]);

const props = defineProps({
  files: {
    type: Array,
    default: () => [],
  },
  currentIndex: {
    type: Number,
    default: -1,
  },
});

// 缩略图缓存
const thumbnails = ref({});

// 搜索关键词
const searchKeyword = ref("");

const fileItems = computed(() => {
  return props.files.map((file, index) => {
    const fileName = file.split(/[\\/]/).pop();
    return {
      index,
      name: fileName,
      path: file,
      isActive: index === props.currentIndex,
    };
  });
});

// 过滤后的文件列表
const filteredFileItems = computed(() => {
  if (!searchKeyword.value.trim()) {
    return fileItems.value;
  }

  const keyword = searchKeyword.value.toLowerCase().trim();
  return fileItems.value.filter((item) => item.name.toLowerCase().includes(keyword));
});

// 过滤后的文件数量
const filteredCount = computed(() => filteredFileItems.value.length);

const selectFile = (index) => {
  emit("select-file", index);
};

const getFileExtension = (filename) => {
  return filename.split(".").pop().toLowerCase();
};

const isImageFile = (filename) => {
  const imageExtensions = ["jpg", "jpeg", "png", "bmp", "gif", "tiff", "webp"];
  return imageExtensions.includes(getFileExtension(filename));
};

// 批量生成缩略图（流式接收）
const generateThumbnails = async () => {
  if (props.files.length === 0) return;

  try {
    // 清空现有缩略图
    thumbnails.value = {};

    // 监听缩略图生成事件
    const { listen } = await import("@tauri-apps/api/event");

    const unlisten = await listen("thumbnail-ready", (event) => {
      const { path, data, index } = event.payload;
      thumbnails.value[path] = data;
    });

    const unlistenComplete = await listen("thumbnails-complete", () => {
      unlisten();
      unlistenComplete();
    });

    // 启动缩略图生成（立即返回，不阻塞）
    await invoke("generate_thumbnails", {
      paths: props.files,
      size: 64,
    });
  } catch (error) {
    console.error("Failed to generate thumbnails:", error);
  }
};

// 监听文件列表变化
watch(
  () => props.files,
  async (newFiles) => {
    if (newFiles.length > 0) {
      await generateThumbnails();
    } else {
      thumbnails.value = {};
    }
  },
  { immediate: true }
);

// 获取缩略图URL
const getThumbnailUrl = (filePath) => {
  return thumbnails.value[filePath] || null;
};
</script>

<template>
  <div class="file-list">
    <div class="file-list-header">
      <h3>图片文件</h3>
      <span class="file-count">{{ files.length }} 个文件</span>
    </div>

    <!-- 搜索框 -->
    <div class="search-box" v-if="files.length > 0">
      <n-input v-model:value="searchKeyword" placeholder="搜索文件名..." clearable size="small">
        <template #prefix>
          <n-icon>
            <svg viewBox="0 0 24 24">
              <path
                fill="currentColor"
                d="M9.5,3A6.5,6.5 0 0,1 16,9.5C16,11.11 15.41,12.59 14.44,13.73L14.71,14H15.5L20.5,19L19,20.5L14,15.5V14.71L13.73,14.44C12.59,15.41 11.11,16 9.5,16A6.5,6.5 0 0,1 3,9.5A6.5,6.5 0 0,1 9.5,3M9.5,5C7,5 5,7 5,9.5C5,12 7,14 9.5,14C12,14 14,12 14,9.5C14,7 12,5 9.5,5Z"
              />
            </svg>
          </n-icon>
        </template>
      </n-input>
      <div v-if="searchKeyword.trim()" class="search-result">找到 {{ filteredCount }} 个文件</div>
    </div>

    <div class="file-list-content">
      <div v-if="files.length === 0" class="empty-state">
        <n-icon size="32">
          <svg viewBox="0 0 24 24">
            <path
              fill="currentColor"
              d="M10 4H4c-1.11 0-2 .89-2 2v12c0 1.11.89 2 2 2h16c1.11 0 2-.89 2-2V8c0-1.11-.89-2-2-2h-8l-2-2z"
            />
          </svg>
        </n-icon>
        <p>请打开包含图片的文件夹</p>
      </div>

      <!-- 无搜索结果提示 -->
      <div v-if="searchKeyword.trim() && filteredCount === 0" class="empty-state">
        <n-icon size="32">
          <svg viewBox="0 0 24 24">
            <path
              fill="currentColor"
              d="M9.5,3A6.5,6.5 0 0,1 16,9.5C16,11.11 15.41,12.59 14.44,13.73L14.71,14H15.5L20.5,19L19,20.5L14,15.5V14.71L13.73,14.44C12.59,15.41 11.11,16 9.5,16A6.5,6.5 0 0,1 3,9.5A6.5,6.5 0 0,1 9.5,3M9.5,5C7,5 5,7 5,9.5C5,12 7,14 9.5,14C12,14 14,12 14,9.5C14,7 12,5 9.5,5Z"
            />
          </svg>
        </n-icon>
        <p>未找到匹配的文件</p>
      </div>

      <div
        v-for="item in filteredFileItems"
        :key="item.index"
        class="file-item"
        :class="{ active: item.isActive }"
        @click="selectFile(item.index)"
      >
        <!-- 缩略图或图标 -->
        <div class="file-thumbnail">
          <img
            v-if="getThumbnailUrl(item.path)"
            :src="getThumbnailUrl(item.path)"
            :alt="item.name"
            class="thumbnail-img"
          />
          <n-icon v-else-if="isImageFile(item.name)" size="40" class="placeholder-icon">
            <svg viewBox="0 0 24 24">
              <path
                fill="currentColor"
                d="M21 19V5c0-1.1-.9-2-2-2H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2zM8.5 13.5l2.5 3.01L14.5 12l4.5 6H5l3.5-4.5z"
              />
            </svg>
          </n-icon>
          <n-icon v-else size="40" class="placeholder-icon">
            <svg viewBox="0 0 24 24">
              <path
                fill="currentColor"
                d="M14,2H6A2,2 0 0,0 4,4V20A2,2 0 0,0 6,22H18A2,2 0 0,0 20,20V8L14,2M18,20H6V4H13V9H18V20Z"
              />
            </svg>
          </n-icon>
        </div>

        <div class="file-info">
          <div class="file-name" :title="item.name">
            {{ item.name }}
          </div>
          <div class="file-index">{{ item.index + 1 }} / {{ files.length }}</div>
        </div>

        <div v-if="item.isActive" class="active-indicator">
          <n-icon size="14">
            <svg viewBox="0 0 24 24">
              <path
                fill="currentColor"
                d="M12,2A10,10 0 0,0 2,12A10,10 0 0,0 12,22A10,10 0 0,0 22,12A10,10 0 0,0 12,2M11,16.5L6.5,12L7.91,10.59L11,13.67L16.59,8.09L18,9.5L11,16.5Z"
              />
            </svg>
          </n-icon>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.file-list {
  height: 100%;
  display: flex;
  flex-direction: column;
  background-color: #ffffff !important;
}

.file-list-header {
  padding: 16px;
  border-bottom: 1px solid #e0e0e0 !important;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.file-list-header h3 {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
  color: #333333 !important;
}

.file-count {
  font-size: 12px;
  color: #666666 !important;
}

/* 搜索框 */
.search-box {
  padding: 8px 16px 12px;
  border-bottom: 1px solid #e0e0e0 !important;
}

.search-result {
  margin-top: 8px;
  font-size: 12px;
  color: #666666 !important;
  text-align: center;
}

.file-list-content {
  flex: 1;
  overflow-y: auto;
  overflow-x: auto; /* 添加横向滚动 */
  padding: 8px;
}

/* 滚动条样式 */
.file-list-content::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

.file-list-content::-webkit-scrollbar-track {
  background: #f5f5f5;
  border-radius: 4px;
}

.file-list-content::-webkit-scrollbar-thumb {
  background-color: #d0d0d0;
  border-radius: 4px;
  border: 2px solid #f5f5f5;
}

.file-list-content::-webkit-scrollbar-thumb:hover {
  background-color: #b0b0b0;
}

/* Firefox 滚动条样式 */
.file-list-content {
  scrollbar-width: thin;
  scrollbar-color: #d0d0d0 #f5f5f5;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 200px;
  color: #999999 !important;
  text-align: center;
}

.empty-state p {
  margin: 8px 0 0 0;
  font-size: 12px;
  color: #999999 !important;
}

.file-item {
  display: flex;
  align-items: center;
  padding: 8px;
  border-radius: 6px;
  cursor: pointer;
  transition: background-color 0.2s;
  margin-bottom: 4px;
  color: #333333 !important;
  gap: 12px;
  min-width: fit-content; /* 让文件项根据内容自动扩展宽度 */
}

.file-item:hover {
  background-color: #f8f9fa !important;
}

.file-thumbnail {
  width: 48px;
  height: 48px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: #f5f5f5;
  border-radius: 4px;
  overflow: hidden;
}

.thumbnail-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.placeholder-icon {
  color: #999 !important;
}

.file-item.active {
  background-color: #e3f2fd !important;
  color: #1976d2 !important;
}

.file-info {
  flex: 1 1 auto; /* 允许扩展和收缩 */
  min-width: 120px; /* 最小宽度，确保有一定的显示空间 */
}

.file-name {
  font-size: 13px;
  line-height: 1.4;
  color: inherit !important;
  white-space: nowrap; /* 不换行，完整显示文件名 */
}

.file-index {
  font-size: 11px;
  color: #999999 !important;
  margin-top: 2px;
}

.file-item.active .file-index {
  color: #1976d2 !important;
}

.active-indicator {
  color: #1976d2 !important;
}
</style>
