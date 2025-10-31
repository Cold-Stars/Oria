<script setup>
import { computed } from "vue";

const emit = defineEmits([
  "switch-tool",
  "open-folder",
  "save",
  "show-help",
  "toggle-annotations",
  "export-coco",
  "export-yolo",
  "import-coco",
  "import-labels",
  "show-inference-settings",
  "inference-one",
  "inference-all",
]);

const props = defineProps({
  currentTool: {
    type: String,
    default: "select",
  },
  saveStatus: {
    type: String,
    default: "saved", // saved, saving, unsaved, error
  },
  lastSaveTime: {
    type: Date,
    default: null,
  },
  showAnnotations: {
    type: Boolean,
    default: true,
  },
  hasImage: {
    type: Boolean,
    default: false,
  },
  inferenceConfigured: {
    type: Boolean,
    default: false,
  },
});

const tools = [
  { id: "select", label: "选择", icon: "🔍" },
  { id: "rectangle", label: "矩形框", icon: "▭" },
  { id: "rotated-rectangle", label: "旋转框", icon: "◇" },
];

// 保存状态文本和图标
const saveStatusInfo = computed(() => {
  switch (props.saveStatus) {
    case "saving":
      return { text: "保存中...", icon: "⏳", color: "#2196F3" };
    case "saved":
      return { text: "已保存", icon: "✓", color: "#4CAF50" };
    case "unsaved":
      return { text: "未保存", icon: "●", color: "#FF9800" };
    case "error":
      return { text: "保存失败", icon: "✗", color: "#F44336" };
    default:
      return { text: "", icon: "", color: "#666" };
  }
});

// 格式化最后保存时间
const formattedSaveTime = computed(() => {
  if (!props.lastSaveTime) return "";

  const now = new Date();
  const diff = Math.floor((now - props.lastSaveTime) / 1000); // 秒

  if (diff < 60) return "刚刚";
  if (diff < 3600) return `${Math.floor(diff / 60)}分钟前`;
  if (diff < 86400) return `${Math.floor(diff / 3600)}小时前`;

  return props.lastSaveTime.toLocaleTimeString("zh-CN", {
    hour: "2-digit",
    minute: "2-digit",
  });
});

// 导入导出下拉菜单选项
const exportOptions = [
  {
    label: "📦 导出COCO",
    key: "export-coco",
  },
  {
    label: "📄 导出YOLO",
    key: "export-yolo",
  },
  {
    type: "divider",
    key: "d1",
  },
  {
    label: "📥 导入COCO",
    key: "import-coco",
  },
  {
    label: "🏷️ 导入标签",
    key: "import-labels",
  },
];

const handleExportSelect = (key) => {
  emit(key);
};

const switchTool = (tool) => {
  emit("switch-tool", tool);
};

const openFolder = () => {
  emit("open-folder");
};

const save = () => {
  emit("save");
};

const showHelp = () => {
  emit("show-help");
};

const toggleAnnotations = () => {
  emit("toggle-annotations");
};

const exportCoco = () => {
  emit("export-coco");
};

const exportYolo = () => {
  emit("export-yolo");
};

const importCoco = () => {
  emit("import-coco");
};

const showInferenceSettings = () => {
  emit("show-inference-settings");
};

const inferenceOne = () => {
  emit("inference-one");
};

const inferenceAll = () => {
  emit("inference-all");
};
</script>

<template>
  <div class="toolbar">
    <!-- 主要操作区 -->
    <div class="toolbar-section primary-actions">
      <n-button type="primary" @click="openFolder" size="small" class="btn-responsive">
        <span class="btn-icon">📁</span>
        <span class="btn-text">打开文件夹</span>
      </n-button>

      <n-button type="success" @click="save" size="small" :disabled="saveStatus === 'saving'" class="btn-responsive">
        <span class="btn-icon">💾</span>
        <span class="btn-text">{{ saveStatus === "saving" ? "保存中..." : "保存" }}</span>
      </n-button>

      <!-- 保存状态指示器 -->
      <div class="save-status" :style="{ color: saveStatusInfo.color }">
        <span class="save-icon">{{ saveStatusInfo.icon }}</span>
        <span class="save-text">{{ saveStatusInfo.text }}</span>
        <span v-if="saveStatus === 'saved' && formattedSaveTime" class="save-time">
          ({{ formattedSaveTime }})
        </span>
      </div>
    </div>

    <n-divider vertical class="divider-responsive" />

    <!-- 导入导出区 -->
    <div class="toolbar-section export-actions">
      <n-dropdown trigger="hover" :options="exportOptions" @select="handleExportSelect">
        <n-button size="small" secondary class="btn-responsive">
          <span class="btn-icon">📦</span>
          <span class="btn-text">导入/导出</span>
        </n-button>
      </n-dropdown>
    </div>

    <n-divider vertical class="divider-responsive" />

    <!-- 推理区 -->
    <div class="toolbar-section inference-actions">
      <n-button @click="showInferenceSettings" size="small" secondary class="btn-responsive">
        <span class="btn-icon">⚙️</span>
        <span class="btn-text">推理设置</span>
      </n-button>

      <n-button
        type="info"
        @click="inferenceOne"
        :disabled="!hasImage || !inferenceConfigured"
        size="small"
        class="btn-responsive"
      >
        <span class="btn-icon">🚀</span>
        <span class="btn-text">当前图</span>
      </n-button>
      <n-button
        type="info"
        @click="inferenceAll"
        :disabled="!hasImage || !inferenceConfigured"
        size="small"
        class="btn-responsive btn-inference-all"
      >
        <span class="btn-icon">🎯</span>
        <span class="btn-text">推理当前及之后</span>
      </n-button>
    </div>

    <n-divider vertical class="divider-responsive" />

    <!-- 工具区 -->
    <div class="toolbar-section tools">
      <div class="tool-group">
        <n-button
          v-for="tool in tools"
          :key="tool.id"
          :type="currentTool === tool.id ? 'primary' : 'default'"
          @click="switchTool(tool.id)"
          size="small"
          class="tool-btn"
          :disabled="!hasImage && tool.id !== 'select'"
        >
          <span class="tool-icon">{{ tool.icon }}</span>
          <span class="tool-label">{{ tool.label }}</span>
        </n-button>
      </div>
    </div>

    <n-divider vertical class="divider-responsive" />

    <!-- 右侧操作区 -->
    <div class="toolbar-section right-actions">
      <n-button
        :type="showAnnotations ? 'primary' : 'default'"
        size="small"
        @click="toggleAnnotations"
        class="btn-responsive"
      >
        <span class="btn-icon">{{ showAnnotations ? "👁️" : "🚫" }}</span>
        <span class="btn-text">{{ showAnnotations ? "显示标注" : "隐藏标注" }}</span>
      </n-button>

      <n-button text size="small" @click="showHelp" class="btn-help">
        ❓
      </n-button>
    </div>
  </div>
</template>

<style scoped>
.toolbar {
  min-height: 50px;
  background-color: #ffffff !important;
  border-bottom: 1px solid #e0e0e0 !important;
  display: flex;
  align-items: center;
  padding: 8px 12px;
  gap: 12px;
  flex-wrap: wrap;
  overflow-x: auto;
}

.toolbar-section {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
}

/* 响应式按钮 */
.btn-responsive {
  display: flex;
  align-items: center;
  gap: 4px;
  white-space: nowrap;
}

.btn-icon {
  flex-shrink: 0;
}

.btn-text {
  flex-shrink: 0;
}

.btn-help {
  min-width: 32px;
}

/* 保存状态 */
.save-status {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  padding: 4px 8px;
  border-radius: 4px;
  background-color: rgba(0, 0, 0, 0.05);
  transition: all 0.3s ease;
  white-space: nowrap;
}

.save-icon {
  font-size: 14px;
  animation: pulse 2s ease-in-out infinite;
}

.save-text {
  font-weight: 500;
}

.save-time {
  font-size: 11px;
  opacity: 0.7;
}

@keyframes pulse {
  0%,
  100% {
    opacity: 1;
  }
  50% {
    opacity: 0.6;
  }
}

/* 工具区 */
.tools {
  flex: 0 1 auto;
}

.tool-group {
  display: flex;
  gap: 4px;
  background-color: #f8f9fa;
  border-radius: 6px;
  padding: 4px;
}

.tool-btn {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 4px;
  min-width: 60px;
  height: auto;
  padding: 6px 8px;
  white-space: nowrap;
}

.tool-icon {
  font-size: 18px;
  line-height: 1.2;
  display: block;
}

.tool-label {
  font-size: 11px;
  line-height: 1.2;
  display: block;
  white-space: nowrap;
}

/* 响应式设计 - 小屏幕 */
@media (max-width: 1200px) {
  .toolbar {
    gap: 8px;
    padding: 8px;
  }

  .toolbar-section {
    gap: 4px;
  }

  .btn-inference-all .btn-text {
    display: none;
  }

  .btn-inference-all .btn-icon::after {
    content: "推理";
    font-size: 12px;
    margin-left: 4px;
  }

  .save-time {
    display: none;
  }
}

@media (max-width: 900px) {
  .divider-responsive {
    display: none;
  }

  .btn-text {
    display: none;
  }

  .btn-responsive {
    min-width: 32px;
    justify-content: center;
  }

  .save-text {
    display: none;
  }

  .tool-btn {
    min-width: 50px;
    padding: 4px 6px;
  }

  .tool-icon {
    font-size: 16px;
  }

  .tool-label {
    font-size: 10px;
  }
}

@media (max-width: 600px) {
  .toolbar {
    padding: 6px;
    gap: 6px;
  }

  .export-actions,
  .inference-actions {
    order: 10;
  }

  .tool-label {
    display: none;
  }

  .tool-btn {
    min-width: 40px;
    padding: 4px;
  }
}
</style>
