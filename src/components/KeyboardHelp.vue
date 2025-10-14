<script setup>
import { ref, computed } from "vue";
import { formatShortcutKey, DEFAULT_SHORTCUTS } from "../utils/keyboard.js";

const emit = defineEmits(["close"]);

const props = defineProps({
  visible: {
    type: Boolean,
    default: false,
  },
});

const showModal = computed({
  get: () => props.visible,
  set: (value) => {
    if (!value) {
      emit("close");
    }
  },
});

// 按类别组织快捷键
const shortcutCategories = computed(() => {
  const categories = {
    文件操作: [],
    工具切换: [],
    导航: [],
    标注操作: [],
    视图操作: [],
    其他: [],
  };

  Object.entries(DEFAULT_SHORTCUTS).forEach(([key, config]) => {
    const formattedKey = formatShortcutKey(key);
    const shortcut = { key: formattedKey, description: config.description };

    if (config.action.includes("open") || config.action.includes("save")) {
      categories["文件操作"].push(shortcut);
    } else if (config.action.includes("Tool")) {
      categories["工具切换"].push(shortcut);
    } else if (
      config.action.includes("Image") ||
      config.action.includes("prev") ||
      config.action.includes("next")
    ) {
      categories["导航"].push(shortcut);
    } else if (
      config.action.includes("Annotation") ||
      config.action.includes("delete") ||
      config.action.includes("select")
    ) {
      categories["标注操作"].push(shortcut);
    } else if (
      config.action.includes("zoom") ||
      config.action.includes("fit") ||
      config.action.includes("reset")
    ) {
      categories["视图操作"].push(shortcut);
    } else {
      categories["其他"].push(shortcut);
    }
  });

  return categories;
});

const close = () => {
  emit("close");
};
</script>

<template>
  <n-modal
    v-model:show="showModal"
    preset="dialog"
    title="键盘快捷键"
    style="width: 600px"
  >
    <div class="keyboard-help">
      <div
        v-for="(shortcuts, category) in shortcutCategories"
        :key="category"
        class="category-section"
      >
        <h3 class="category-title">{{ category }}</h3>
        <div class="shortcuts-list">
          <div
            v-for="shortcut in shortcuts"
            :key="shortcut.key"
            class="shortcut-item"
          >
            <div class="shortcut-key">
              {{ shortcut.key }}
            </div>
            <div class="shortcut-description">
              {{ shortcut.description }}
            </div>
          </div>
        </div>
      </div>
    </div>

    <template #action>
      <n-button @click="close"> 关闭 </n-button>
    </template>
  </n-modal>
</template>

<style scoped>
.keyboard-help {
  max-height: 500px;
  overflow-y: auto;
  background-color: #ffffff !important;
}

.category-section {
  margin-bottom: 24px;
}

.category-section:last-child {
  margin-bottom: 0;
}

.category-title {
  margin: 0 0 12px 0;
  font-size: 16px;
  font-weight: 600;
  color: #333333 !important;
  border-bottom: 1px solid #e0e0e0 !important;
  padding-bottom: 8px;
}

.shortcuts-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.shortcut-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  background-color: #f8f9fa !important;
  border-radius: 6px;
  transition: background-color 0.2s;
}

.shortcut-item:hover {
  background-color: #e9ecef !important;
}

.shortcut-key {
  font-family: "Courier New", monospace;
  font-size: 13px;
  font-weight: 600;
  color: #495057 !important;
  background-color: #ffffff !important;
  padding: 4px 8px;
  border-radius: 4px;
  border: 1px solid #dee2e6 !important;
  min-width: 120px;
  text-align: center;
}

.shortcut-description {
  font-size: 14px;
  color: #666666 !important;
  flex: 1;
  margin-left: 16px;
}
</style>
