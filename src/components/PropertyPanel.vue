<script setup>
import { ref, computed, watch, nextTick } from "vue";

const emit = defineEmits([
  "update-annotation",
  "delete-annotation",
  "select-annotation",
]);

const props = defineProps({
  selectedAnnotation: {
    type: Object,
    default: null,
  },
  annotations: {
    type: Array,
    default: () => [],
  },
  labelCategories: {
    type: Array,
    default: () => [],
  },
  categoryColors: {
    type: Object,
    default: () => ({}),
  },
});

// 编辑状态
const editingLabel = ref("");
const editingClass = ref("");
const labelInputRef = ref(null);

// 计算属性 - 标签选项
const labelOptions = computed(() => {
  const options = props.labelCategories.map((category) => ({
    label: category,
    value: category,
  }));

  // 始终返回所有选项，即使输入为空
  // n-auto-complete 会根据输入自动过滤
  return options;
});

// 计算属性
const annotationList = computed(() => {
  return props.annotations.map((annotation, index) => ({
    ...annotation,
    index: index + 1,
    displayName: annotation.label || `标注 ${index + 1}`,
  }));
});

const selectedAnnotationInfo = computed(() => {
  if (!props.selectedAnnotation) return null;

  return {
    ...props.selectedAnnotation,
    area: Math.round(
      props.selectedAnnotation.width * props.selectedAnnotation.height
    ),
    centerX: Math.round(
      props.selectedAnnotation.x + props.selectedAnnotation.width / 2
    ),
    centerY: Math.round(
      props.selectedAnnotation.y + props.selectedAnnotation.height / 2
    ),
  };
});

// 方法
const updateAnnotation = (updates) => {
  if (props.selectedAnnotation) {
    emit("update-annotation", props.selectedAnnotation.id, updates);
  }
};

const deleteAnnotation = () => {
  if (props.selectedAnnotation) {
    emit("delete-annotation", props.selectedAnnotation.id);
  }
};

const updateLabelImmediate = (newLabel) => {
  if (!props.selectedAnnotation) {
    return;
  }

  if (!newLabel) {
    return;
  }

  const trimmedLabel = newLabel.trim();
  const currentLabel = props.selectedAnnotation.label || "";

  if (trimmedLabel && trimmedLabel !== currentLabel) {
    updateAnnotation({ label: trimmedLabel });
  }
};

const updateLabel = () => {
  updateLabelImmediate(editingLabel.value || "");

  // 更新后失去焦点，以便快捷键可以正常工作
  nextTick(() => {
    if (labelInputRef.value && labelInputRef.value.blur) {
      labelInputRef.value.blur();
    }
  });
};

// 当选择下拉选项时自动更新
const onLabelSelect = (value) => {
  updateLabelImmediate(value || "");

  // 选择后失去焦点
  nextTick(() => {
    if (labelInputRef.value && labelInputRef.value.blur) {
      labelInputRef.value.blur();
    }
  });
};

// 输入框失去焦点时自动保存
const onLabelBlur = () => {
  if (!props.selectedAnnotation) return;

  updateLabelImmediate(editingLabel.value || "");
};

const selectClass = (className) => {
  updateAnnotation({ label: className });
};

const selectAnnotation = (annotation) => {
  emit("select-annotation", annotation);
};

// 切换标注的显示/隐藏状态
const toggleAnnotationVisibility = (annotation) => {
  const newVisible = annotation.visible === false ? true : false;
  emit("update-annotation", annotation.id, { visible: newVisible });
};

// 监听选中标注变化
const onSelectedAnnotationChange = (newAnnotation, oldAnnotation) => {
  if (newAnnotation) {
    editingLabel.value = newAnnotation.label || "";
    editingClass.value = newAnnotation.label || "";
  } else {
    editingLabel.value = "";
    editingClass.value = "";
  }
};

// 监听器 - 深度监听确保属性变化也能捕获
watch(() => props.selectedAnnotation, onSelectedAnnotationChange, {
  immediate: true,
  deep: true,
});

// 额外监听label属性的变化
watch(
  () => props.selectedAnnotation?.label,
  (newLabel) => {
    if (props.selectedAnnotation && newLabel !== undefined) {
      editingLabel.value = newLabel || "";
    }
  }
);
</script>

<template>
  <div class="property-panel">
    <!-- 标注列表 -->
    <div class="panel-section">
      <div class="section-header">
        <h3>标注列表</h3>
        <span class="annotation-count">{{ annotations.length }}</span>
      </div>

      <div class="annotation-list">
        <div v-if="annotations.length === 0" class="empty-state">
          <n-icon size="24">
            <svg viewBox="0 0 24 24">
              <path
                fill="currentColor"
                d="M5.5,9A1.5,1.5 0 0,0 7,7.5A1.5,1.5 0 0,0 5.5,6A1.5,1.5 0 0,0 4,7.5A1.5,1.5 0 0,0 5.5,9M17.41,11.58C17.77,11.94 18,12.44 18,13C18,13.55 17.78,14.05 17.41,14.41L12.41,19.41C12.05,19.77 11.55,20 11,20C10.45,20 9.95,19.78 9.59,19.41L2.59,12.41C2.22,12.05 2,11.55 2,11V4C2,2.89 2.89,2 4,2H11C11.55,2 12.05,2.22 12.41,2.59L19.41,9.59C19.77,9.95 20,10.45 20,11C20,11.55 19.78,12.05 19.41,12.41L14.41,17.41C14.05,17.77 13.55,18 13,18C12.45,18 11.95,17.78 11.59,17.41L17.41,11.58Z"
              />
            </svg>
          </n-icon>
          <p>暂无标注</p>
        </div>

        <div
          v-for="annotation in annotationList"
          :key="annotation.id"
          class="annotation-item"
          :class="{
            active: selectedAnnotation?.id === annotation.id,
            hidden: annotation.visible === false,
          }"
          @click="selectAnnotation(annotation)"
        >
          <div class="annotation-info">
            <div class="annotation-name">
              <span class="annotation-icon">
                {{ annotation.type === "rectangle" ? "▭" : "◇" }}
              </span>
              {{ annotation.displayName }}
            </div>
            <div class="annotation-details">
              <span class="type-badge" :class="annotation.type">
                {{ annotation.type === "rectangle" ? "矩形" : "旋转矩形" }}
              </span>
              <span class="coordinates">
                {{ Math.round(annotation.x) }}, {{ Math.round(annotation.y) }}
              </span>
            </div>
          </div>

          <div class="annotation-actions">
            <n-button
              text
              size="small"
              @click.stop="toggleAnnotationVisibility(annotation)"
              :title="annotation.visible === false ? '显示' : '隐藏'"
            >
              <n-icon size="14">
                <svg viewBox="0 0 24 24" v-if="annotation.visible !== false">
                  <path
                    fill="currentColor"
                    d="M12,9A3,3 0 0,0 9,12A3,3 0 0,0 12,15A3,3 0 0,0 15,12A3,3 0 0,0 12,9M12,17A5,5 0 0,1 7,12A5,5 0 0,1 12,7A5,5 0 0,1 17,12A5,5 0 0,1 12,17M12,4.5C7,4.5 2.73,7.61 1,12C2.73,16.39 7,19.5 12,19.5C17,19.5 21.27,16.39 23,12C21.27,7.61 17,4.5 12,4.5Z"
                  />
                </svg>
                <svg viewBox="0 0 24 24" v-else>
                  <path
                    fill="currentColor"
                    d="M11.83,9L15,12.16C15,12.11 15,12.05 15,12A3,3 0 0,0 12,9C11.94,9 11.89,9 11.83,9M7.53,9.8L9.08,11.35C9.03,11.56 9,11.77 9,12A3,3 0 0,0 12,15C12.22,15 12.44,14.97 12.65,14.92L14.2,16.47C13.53,16.8 12.79,17 12,17A5,5 0 0,1 7,12C7,11.21 7.2,10.47 7.53,9.8M2,4.27L4.28,6.55L4.73,7C3.08,8.3 1.78,10 1,12C2.73,16.39 7,19.5 12,19.5C13.55,19.5 15.03,19.2 16.38,18.66L16.81,19.08L19.73,22L21,20.73L3.27,3M12,7A5,5 0 0,1 17,12C17,12.64 16.87,13.26 16.64,13.82L19.57,16.75C21.07,15.5 22.27,13.86 23,12C21.27,7.61 17,4.5 12,4.5C10.6,4.5 9.26,4.75 8,5.2L10.17,7.35C10.74,7.13 11.35,7 12,7Z"
                  />
                </svg>
              </n-icon>
            </n-button>
            <n-button
              text
              size="small"
              @click.stop="emit('delete-annotation', annotation.id)"
            >
              <n-icon size="14">
                <svg viewBox="0 0 24 24">
                  <path
                    fill="currentColor"
                    d="M19,4H15.5L14.5,3H9.5L8.5,4H5V6H19M6,19A2,2 0 0,0 8,21H16A2,2 0 0,0 18,19V7H6V19Z"
                  />
                </svg>
              </n-icon>
            </n-button>
          </div>
        </div>
      </div>
    </div>

    <!-- 属性编辑 -->
    <div v-if="selectedAnnotation" class="panel-section">
      <div class="section-header">
        <h3>属性编辑</h3>
      </div>

      <div class="property-form">
        <!-- 标签编辑 -->
        <div class="form-group">
          <label>标签</label>
          <div class="input-group">
            <n-auto-complete
              ref="labelInputRef"
              v-model:value="editingLabel"
              :options="labelOptions"
              placeholder="输入或选择标签"
              size="small"
              clearable
              :get-show="() => labelOptions.length > 0"
              @select="onLabelSelect"
              @blur="onLabelBlur"
            >
              <template
                #default="{ handleInput, handleBlur, handleFocus, value }"
              >
                <n-input
                  :value="value"
                  placeholder="输入或选择标签"
                  size="small"
                  @input="handleInput"
                  @focus="handleFocus"
                  @blur="handleBlur"
                  @keydown.enter.prevent="updateLabel"
                />
              </template>
            </n-auto-complete>
          </div>
        </div>

        <!-- 标签类别 -->
        <div class="form-group">
          <label>标签类别 ({{ labelCategories.length }})</label>
          <div v-if="labelCategories.length === 0" class="empty-categories">
            <p>暂无类别，输入标签后自动添加</p>
          </div>
          <div v-else class="class-grid">
            <n-button
              v-for="className in labelCategories"
              :key="className"
              :type="
                selectedAnnotation.label === className ? 'primary' : 'default'
              "
              size="small"
              @click="selectClass(className)"
              class="class-btn"
              :style="{
                borderLeft: `4px solid ${
                  categoryColors[className] || '#00aaff'
                }`,
              }"
            >
              {{ className }}
            </n-button>
          </div>
        </div>

        <!-- 位置信息 -->
        <div class="form-group">
          <label>位置信息</label>
          <div class="info-grid">
            <div class="info-item">
              <span class="info-label">X:</span>
              <span class="info-value">{{
                Math.round(selectedAnnotation.x)
              }}</span>
            </div>
            <div class="info-item">
              <span class="info-label">Y:</span>
              <span class="info-value">{{
                Math.round(selectedAnnotation.y)
              }}</span>
            </div>
            <div class="info-item">
              <span class="info-label">宽:</span>
              <span class="info-value">{{
                Math.round(selectedAnnotation.width)
              }}</span>
            </div>
            <div class="info-item">
              <span class="info-label">高:</span>
              <span class="info-value">{{
                Math.round(selectedAnnotation.height)
              }}</span>
            </div>
            <div class="info-item">
              <span class="info-label">面积:</span>
              <span class="info-value">{{ selectedAnnotationInfo.area }}</span>
            </div>
            <div
              v-if="selectedAnnotation.rotation !== undefined"
              class="info-item"
            >
              <span class="info-label">角度:</span>
              <span class="info-value"
                >{{
                  Math.round((selectedAnnotation.rotation * 180) / Math.PI)
                }}°</span
              >
            </div>
          </div>
        </div>

        <!-- 操作按钮 -->
        <div class="form-group">
          <n-button
            type="error"
            size="small"
            @click="deleteAnnotation"
            class="delete-btn"
          >
            <n-icon size="14">
              <svg viewBox="0 0 24 24">
                <path
                  fill="currentColor"
                  d="M19,4H15.5L14.5,3H9.5L8.5,4H5V6H19M6,19A2,2 0 0,0 8,21H16A2,2 0 0,0 18,19V7H6V19Z"
                />
              </svg>
            </n-icon>
            删除标注
          </n-button>
        </div>
      </div>
    </div>

    <!-- 统计信息 -->
    <div class="panel-section">
      <div class="section-header">
        <h3>统计信息</h3>
      </div>

      <div class="stats">
        <div class="stat-item">
          <span class="stat-label">总标注数:</span>
          <span class="stat-value">{{ annotations.length }}</span>
        </div>
        <div class="stat-item">
          <span class="stat-label">矩形框:</span>
          <span class="stat-value">
            {{ annotations.filter((a) => a.type === "rectangle").length }}
          </span>
        </div>
        <div class="stat-item">
          <span class="stat-label">旋转框:</span>
          <span class="stat-value">
            {{
              annotations.filter((a) => a.type === "rotated-rectangle").length
            }}
          </span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.property-panel {
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow-y: auto;
}

.panel-section {
  border-bottom: 1px solid #e0e0e0;
  padding: 16px;
}

.panel-section:last-child {
  border-bottom: none;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.section-header h3 {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
  color: #333;
}

.annotation-count {
  font-size: 12px;
  color: #666;
  background-color: #f0f0f0;
  padding: 2px 6px;
  border-radius: 10px;
}

.annotation-list {
  max-height: 300px;
  overflow-y: auto;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100px;
  color: #999;
  text-align: center;
}

.empty-state p {
  margin: 8px 0 0 0;
  font-size: 12px;
}

.annotation-item {
  display: flex;
  align-items: center;
  padding: 8px;
  border-radius: 6px;
  margin-bottom: 4px;
  transition: background-color 0.2s;
  cursor: pointer;
  border: 1px solid transparent;
}

.annotation-item:hover {
  background-color: #f8f9fa;
  border-color: #ddd;
  transform: translateY(-1px);
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.annotation-item.active {
  background-color: #e3f2fd;
  border-color: #1976d2;
  box-shadow: 0 2px 8px rgba(25, 118, 210, 0.2);
}

.annotation-item.hidden {
  opacity: 0.5;
  background-color: #f5f5f5;
}

.annotation-item.hidden .annotation-name {
  text-decoration: line-through;
  color: #999;
}

.annotation-info {
  flex: 1;
  min-width: 0;
}

.annotation-name {
  font-size: 13px;
  font-weight: 500;
  margin-bottom: 2px;
  display: flex;
  align-items: center;
  gap: 6px;
}

.annotation-icon {
  font-size: 14px;
  color: #666;
  min-width: 16px;
}

.annotation-details {
  display: flex;
  align-items: center;
  gap: 8px;
}

.type-badge {
  font-size: 10px;
  padding: 2px 6px;
  border-radius: 3px;
  background-color: #e0e0e0;
  color: #666;
}

.type-badge.rectangle {
  background-color: #e3f2fd;
  color: #1976d2;
}

.type-badge.rotated-rectangle {
  background-color: #f3e5f5;
  color: #7b1fa2;
}

.coordinates {
  font-size: 11px;
  color: #999;
}

.annotation-actions {
  margin-left: 8px;
}

.property-form {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.form-group label {
  font-size: 12px;
  font-weight: 500;
  color: #666;
}

.input-group {
  display: flex;
  gap: 6px;
}

.class-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 4px;
}

.class-btn {
  font-size: 11px;
  text-align: center;
  position: relative;
  overflow: hidden;
}

.empty-categories {
  text-align: center;
  padding: 20px;
  color: #999;
  font-size: 12px;
}

.empty-categories p {
  margin: 0;
}

.info-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 8px;
}

.info-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 6px 8px;
  background-color: #f8f9fa;
  border-radius: 4px;
  font-size: 11px;
}

.info-label {
  font-weight: 500;
  color: #666;
}

.info-value {
  color: #333;
}

.delete-btn {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
}

.stats {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.stat-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 12px;
}

.stat-label {
  color: #666;
}

.stat-value {
  font-weight: 500;
  color: #333;
}

/* 强制浅色主题样式 */
.property-panel {
  background-color: #ffffff;
}

.panel-section {
  border-color: #e0e0e0;
}
</style>
