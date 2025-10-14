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
    <div class="toolbar-section">
      <n-button type="primary" @click="openFolder" size="small"> 打开文件夹 </n-button>

      <n-button type="success" @click="save" size="small" :disabled="saveStatus === 'saving'">
        {{ saveStatus === "saving" ? "保存中..." : "保存" }}
      </n-button>

      <n-divider vertical />

      <n-button @click="exportCoco" size="small" secondary> 📦 导出COCO </n-button>

      <n-button @click="exportYolo" size="small" secondary> 📄 导出YOLO </n-button>

      <n-button @click="importCoco" size="small" secondary> 📥 导入COCO </n-button>

      <n-divider vertical />

      <n-button @click="showInferenceSettings" size="small" secondary> ⚙️ 推理设置 </n-button>

      <n-divider vertical />

      <n-space :size="8">
        <n-button
          type="info"
          @click="inferenceOne"
          :disabled="!hasImage || !inferenceConfigured"
          size="small"
          style="min-width: 88px"
        >
          🚀 当前图
        </n-button>
        <n-button
          type="info"
          @click="inferenceAll"
          :disabled="!hasImage || !inferenceConfigured"
          size="small"
          style="min-width: 130px"
        >
          🎯 推理当前及之后
        </n-button>
      </n-space>

      <!-- 保存状态指示器 -->
      <div class="save-status" :style="{ color: saveStatusInfo.color }">
        <span class="save-icon">{{ saveStatusInfo.icon }}</span>
        <span class="save-text">{{ saveStatusInfo.text }}</span>
        <span v-if="saveStatus === 'saved' && formattedSaveTime" class="save-time">
          ({{ formattedSaveTime }})
        </span>
      </div>
    </div>

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

    <div class="toolbar-section">
      <n-button
        :type="showAnnotations ? 'primary' : 'default'"
        size="small"
        @click="toggleAnnotations"
        class="tooltip"
        :data-tooltip="showAnnotations ? '隐藏标注' : '显示标注'"
      >
        {{ showAnnotations ? "👁️ 显示标注" : "🚫 隐藏标注" }}
      </n-button>

      <n-button text size="small" @click="showHelp" class="tooltip" data-tooltip="快捷键帮助 (F1)">
        ❓
      </n-button>
    </div>
  </div>
</template>

<style scoped>
.toolbar {
  height: 50px;
  background-color: #ffffff !important;
  border-bottom: 1px solid #e0e0e0 !important;
  display: flex;
  align-items: center;
  padding: 0 16px;
  gap: 24px;
}

.toolbar-section {
  display: flex;
  align-items: center;
  gap: 8px;
}

.save-status {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  padding: 4px 8px;
  border-radius: 4px;
  background-color: rgba(0, 0, 0, 0.05);
  transition: all 0.3s ease;
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

.tools {
  flex: 1;
  justify-content: center;
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
  min-width: 70px;
  height: auto;
  padding: 8px 12px;
  white-space: nowrap;
}

.tool-icon {
  font-size: 20px;
  line-height: 1.2;
  display: block;
}

.tool-label {
  font-size: 12px;
  line-height: 1.2;
  display: block;
  white-space: nowrap;
}
</style>
