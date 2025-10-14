<script setup>
import { ref, computed, onMounted, onUnmounted, watch, nextTick } from "vue";

const emit = defineEmits([
  "add-annotation",
  "update-annotation",
  "select-annotation",
  "switch-tool",
]);

const props = defineProps({
  image: {
    type: Object,
    default: null,
  },
  annotations: {
    type: Array,
    default: () => [],
  },
  selectedAnnotation: {
    type: Object,
    default: null,
  },
  currentTool: {
    type: String,
    default: "select",
  },
  categoryColors: {
    type: Object,
    default: () => ({}),
  },
  labelCategories: {
    type: Array,
    default: () => [],
  },
  showAnnotations: {
    type: Boolean,
    default: true,
  },
});

// 计算属性 - 标签选项
const labelOptions = computed(() => {
  const options = props.labelCategories.map((category) => ({
    label: category,
    value: category,
  }));

  // 始终返回所有选项，即使输入为空
  return options;
});

// 画布相关
const canvasContainer = ref(null);
const canvas = ref(null);
const ctx = ref(null);
const imageElement = ref(null);

// 视图状态
const scale = ref(1);
const offsetX = ref(0);
const offsetY = ref(0);
const containerWidth = ref(0);
const containerHeight = ref(0);

// 绘制状态
const isDrawing = ref(false);
const isDragging = ref(false);
const isResizing = ref(false);
const isRotating = ref(false);
const isPanning = ref(false); // 画布拖动状态
const startPoint = ref({ x: 0, y: 0 });
const currentAnnotation = ref(null);
const dragStart = ref({ x: 0, y: 0, offsetX: 0, offsetY: 0 });
const resizeHandle = ref(null); // 当前调整大小的控制点
const rotationHandle = ref(null); // 旋转控制点

// 鼠标位置
const mousePos = ref({ x: 0, y: 0 });

// 浮动标签选择器
const showLabelSelector = ref(false);
const labelSelectorPos = ref({ x: 0, y: 0 });
const labelSelectorAnnotation = ref(null);
const selectedLabel = ref("");

// 初始化画布
const initCanvas = () => {
  if (!canvas.value) return;

  const container = canvasContainer.value;
  containerWidth.value = container.clientWidth;
  containerHeight.value = container.clientHeight;

  canvas.value.width = containerWidth.value;
  canvas.value.height = containerHeight.value;
  ctx.value = canvas.value.getContext("2d");

  draw();
};

// 坐标转换
const screenToCanvas = (screenX, screenY) => {
  const rect = canvas.value.getBoundingClientRect();
  return {
    x: (screenX - rect.left - offsetX.value) / scale.value,
    y: (screenY - rect.top - offsetY.value) / scale.value,
  };
};

const canvasToScreen = (canvasX, canvasY) => {
  return {
    x: canvasX * scale.value + offsetX.value,
    y: canvasY * scale.value + offsetY.value,
  };
};

// 绘制函数
const draw = () => {
  if (!ctx.value || !canvas.value) return;

  // 清空画布
  ctx.value.clearRect(0, 0, canvas.value.width, canvas.value.height);

  // 绘制背景
  ctx.value.fillStyle = "#f5f5f5";
  ctx.value.fillRect(0, 0, canvas.value.width, canvas.value.height);

  // 绘制图片
  if (imageElement.value && props.image) {
    // 使用原始图片尺寸作为画布逻辑尺寸（标注坐标基于此）
    const imgWidth = props.image.width;
    const imgHeight = props.image.height;

    ctx.value.save();
    ctx.value.translate(offsetX.value, offsetY.value);
    ctx.value.scale(scale.value, scale.value);

    // 直接将图片（可能是预览图）绘制到原始尺寸
    // Canvas 会自动将预览图拉伸到目标尺寸
    ctx.value.drawImage(imageElement.value, 0, 0, imgWidth, imgHeight);

    // 绘制标注（根据 showAnnotations 状态）
    // 标注坐标基于原始图片尺寸，与图片绘制尺寸一致
    if (props.showAnnotations) {
      drawAnnotations();
    }

    // 绘制当前正在绘制的标注（总是显示）
    if (isDrawing.value && currentAnnotation.value) {
      drawAnnotation(currentAnnotation.value, true);
    }

    ctx.value.restore();
  }
};

// 检查鼠标是否悬停在标注上
const hoveredAnnotation = ref(null);

const drawAnnotations = () => {
  props.annotations.forEach((annotation) => {
    // 跳过隐藏的标注
    if (annotation.visible === false) return;

    const isSelected = props.selectedAnnotation?.id === annotation.id;
    const isHovered = hoveredAnnotation.value?.id === annotation.id;
    drawAnnotation(annotation, false, isSelected, isHovered);
  });
};

const drawAnnotation = (annotation, isDrawing = false, isSelected = false, isHovered = false) => {
  if (!ctx.value) return;

  ctx.value.save();

  // 获取类别颜色
  const categoryColor =
    annotation.label && props.categoryColors[annotation.label]
      ? props.categoryColors[annotation.label]
      : "#00aaff"; // 默认蓝色

  // 设置样式
  if (isSelected) {
    ctx.value.strokeStyle = "#ff4444"; // 选中时使用红色
    ctx.value.lineWidth = 3;
  } else if (isHovered) {
    ctx.value.strokeStyle = "#ff8800"; // 橙色表示悬停
    ctx.value.lineWidth = 2;
  } else if (isDrawing) {
    ctx.value.strokeStyle = "#44ff44"; // 绘制中使用绿色
    ctx.value.lineWidth = 2;
  } else {
    ctx.value.strokeStyle = categoryColor; // 使用类别颜色
    ctx.value.lineWidth = 2;
  }

  // 填充色也使用类别颜色（半透明）
  // 将 hsl(h, s%, l%) 转换为 hsla(h, s%, l%, 0.1)
  let fillColor;
  if (categoryColor.startsWith("hsl(")) {
    fillColor = categoryColor.replace("hsl(", "hsla(").replace(")", ", 0.1)");
  } else if (categoryColor.startsWith("#")) {
    // 如果是hex颜色，添加透明度
    fillColor = categoryColor + "20";
  } else {
    fillColor = "rgba(0, 170, 255, 0.1)";
  }
  ctx.value.fillStyle = fillColor;

  if (annotation.type === "rectangle") {
    // 绘制矩形
    ctx.value.beginPath();
    ctx.value.rect(annotation.x, annotation.y, annotation.width, annotation.height);
    ctx.value.fill();
    ctx.value.stroke();

    // 绘制控制点（选中或悬停时显示）
    if (isSelected || isHovered) {
      drawControlPoints(annotation, isSelected);
    }

    // 绘制标签（普通矩形）
    if (annotation.label) {
      ctx.value.font = "bold 12px Arial";
      const textMetrics = ctx.value.measureText(annotation.label);
      const textWidth = textMetrics.width;
      const textHeight = 16;
      const padding = 4;

      // 背景矩形
      ctx.value.fillStyle = isSelected ? "#ff4444" : categoryColor;
      ctx.value.fillRect(
        annotation.x,
        annotation.y - textHeight - padding,
        textWidth + padding * 2,
        textHeight + padding
      );

      // 标签文字（白色）
      ctx.value.fillStyle = "#ffffff";
      ctx.value.fillText(annotation.label, annotation.x + padding, annotation.y - padding - 2);
    }
  } else if (annotation.type === "rotated-rectangle") {
    // 绘制旋转矩形
    const centerX = annotation.x + annotation.width / 2;
    const centerY = annotation.y + annotation.height / 2;

    ctx.value.translate(centerX, centerY);
    ctx.value.rotate(annotation.rotation || 0);
    ctx.value.translate(-annotation.width / 2, -annotation.height / 2);

    ctx.value.beginPath();
    ctx.value.rect(0, 0, annotation.width, annotation.height);
    ctx.value.fill();
    ctx.value.stroke();

    // 绘制旋转控制点（选中或悬停时显示）
    if (isSelected || isHovered) {
      drawRotatedControlPoints(annotation, isSelected);
    }

    // 绘制标签（旋转矩形，在变换后的坐标系中）
    if (annotation.label) {
      ctx.value.font = "bold 12px Arial";
      const textMetrics = ctx.value.measureText(annotation.label);
      const textWidth = textMetrics.width;
      const textHeight = 16;
      const padding = 4;

      // 背景矩形（在局部坐标系中，矩形左上角是 (0, 0)）
      ctx.value.fillStyle = isSelected ? "#ff4444" : categoryColor;
      ctx.value.fillRect(0, -textHeight - padding, textWidth + padding * 2, textHeight + padding);

      // 标签文字（白色）
      ctx.value.fillStyle = "#ffffff";
      ctx.value.fillText(annotation.label, padding, -padding - 2);
    }
  }

  ctx.value.restore();
};

const drawControlPoints = (annotation, isSelected = true) => {
  if (!ctx.value) return;

  // 根据选中状态调整透明度
  const opacity = isSelected ? 1.0 : 0.7;

  // 四个角点
  const cornerPoints = [
    { x: annotation.x, y: annotation.y, type: "corner", handle: "nw" },
    {
      x: annotation.x + annotation.width,
      y: annotation.y,
      type: "corner",
      handle: "ne",
    },
    {
      x: annotation.x + annotation.width,
      y: annotation.y + annotation.height,
      type: "corner",
      handle: "se",
    },
    {
      x: annotation.x,
      y: annotation.y + annotation.height,
      type: "corner",
      handle: "sw",
    },
  ];

  // 边中点
  const edgePoints = [
    {
      x: annotation.x + annotation.width / 2,
      y: annotation.y,
      type: "edge",
      handle: "n",
    },
    {
      x: annotation.x + annotation.width,
      y: annotation.y + annotation.height / 2,
      type: "edge",
      handle: "e",
    },
    {
      x: annotation.x + annotation.width / 2,
      y: annotation.y + annotation.height,
      type: "edge",
      handle: "s",
    },
    {
      x: annotation.x,
      y: annotation.y + annotation.height / 2,
      type: "edge",
      handle: "w",
    },
  ];

  // 设置透明度
  ctx.value.globalAlpha = opacity;

  // 绘制角点 (较大，用于调整大小)
  ctx.value.fillStyle = "#ff4444";
  cornerPoints.forEach((point) => {
    ctx.value.beginPath();
    ctx.value.arc(point.x, point.y, 10, 0, Math.PI * 2);
    ctx.value.fill();
  });

  // 绘制边中点 (较小，用于单方向调整)
  ctx.value.fillStyle = "#ff6666";
  edgePoints.forEach((point) => {
    ctx.value.beginPath();
    ctx.value.arc(point.x, point.y, 6, 0, Math.PI * 2);
    ctx.value.fill();
  });

  // 重置透明度
  ctx.value.globalAlpha = 1.0;
};

const drawRotatedControlPoints = (annotation, isSelected = true) => {
  // 旋转矩形的控制点绘制
  if (!ctx.value) return;

  // 根据选中状态调整透明度
  const opacity = isSelected ? 1.0 : 0.7;
  ctx.value.globalAlpha = opacity;

  // 四个角点 (调整大小)
  const cornerPoints = [
    { x: 0, y: 0, type: "corner", handle: "nw" },
    { x: annotation.width, y: 0, type: "corner", handle: "ne" },
    { x: annotation.width, y: annotation.height, type: "corner", handle: "se" },
    { x: 0, y: annotation.height, type: "corner", handle: "sw" },
  ];

  // 边中点 (单方向调整)
  const edgePoints = [
    { x: annotation.width / 2, y: 0, type: "edge", handle: "n" },
    {
      x: annotation.width,
      y: annotation.height / 2,
      type: "edge",
      handle: "e",
    },
    {
      x: annotation.width / 2,
      y: annotation.height,
      type: "edge",
      handle: "s",
    },
    { x: 0, y: annotation.height / 2, type: "edge", handle: "w" },
  ];

  // 绘制角点
  ctx.value.fillStyle = "#ff4444";
  cornerPoints.forEach((point) => {
    ctx.value.beginPath();
    ctx.value.arc(point.x, point.y, 10, 0, Math.PI * 2);
    ctx.value.fill();
  });

  // 绘制边中点
  ctx.value.fillStyle = "#ff6666";
  edgePoints.forEach((point) => {
    ctx.value.beginPath();
    ctx.value.arc(point.x, point.y, 6, 0, Math.PI * 2);
    ctx.value.fill();
  });

  // 旋转控制点 (在顶部中心上方，局部坐标系)
  ctx.value.fillStyle = "#00ff00";
  ctx.value.beginPath();
  ctx.value.arc(annotation.width / 2, -25, 10, 0, Math.PI * 2);
  ctx.value.fill();

  // 旋转控制点连接线
  ctx.value.strokeStyle = "#00ff00";
  ctx.value.lineWidth = 3;
  ctx.value.beginPath();
  ctx.value.moveTo(annotation.width / 2, 0);
  ctx.value.lineTo(annotation.width / 2, -25);
  ctx.value.stroke();

  // 重置透明度
  ctx.value.globalAlpha = 1.0;
};

// 事件处理
// 旧的onMouseDown函数已被onMouseDownFixed替代

const onMouseMove = (event) => {
  const pos = screenToCanvas(event.clientX, event.clientY);
  mousePos.value = pos;

  // 更新悬停状态（拖动画布时不更新）
  if (!isPanning.value) {
    updateHoverState(event);
  }

  // 更新鼠标光标样式
  updateCursor(event);

  // 画布拖动
  if (isPanning.value) {
    const dx = event.clientX - dragStart.value.x;
    const dy = event.clientY - dragStart.value.y;

    offsetX.value = dragStart.value.offsetX + dx;
    offsetY.value = dragStart.value.offsetY + dy;

    draw();
    return; // 拖动画布时不处理其他操作
  }

  if (isDrawing.value && currentAnnotation.value) {
    // 更新当前绘制的标注
    const width = pos.x - startPoint.value.x;
    const height = pos.y - startPoint.value.y;

    currentAnnotation.value.width = Math.abs(width);
    currentAnnotation.value.height = Math.abs(height);

    if (width < 0) {
      currentAnnotation.value.x = startPoint.value.x + width;
    }
    if (height < 0) {
      currentAnnotation.value.y = startPoint.value.y + height;
    }

    draw();
  } else if (isResizing.value && resizeHandle.value && props.selectedAnnotation) {
    // 调整标注大小
    const dx = pos.x - dragStart.value.x;
    const dy = pos.y - dragStart.value.y;
    const originalAnnotation = dragStart.value.annotation;

    let newProps = { ...originalAnnotation };

    if (originalAnnotation.type === "rectangle") {
      // 矩形框的调整逻辑
      switch (resizeHandle.value.handle) {
        case "nw": // 左上角
          newProps.x = originalAnnotation.x + dx;
          newProps.y = originalAnnotation.y + dy;
          newProps.width = originalAnnotation.width - dx;
          newProps.height = originalAnnotation.height - dy;
          break;
        case "ne": // 右上角
          newProps.y = originalAnnotation.y + dy;
          newProps.width = originalAnnotation.width + dx;
          newProps.height = originalAnnotation.height - dy;
          break;
        case "se": // 右下角
          newProps.width = originalAnnotation.width + dx;
          newProps.height = originalAnnotation.height + dy;
          break;
        case "sw": // 左下角
          newProps.x = originalAnnotation.x + dx;
          newProps.width = originalAnnotation.width - dx;
          newProps.height = originalAnnotation.height + dy;
          break;
        case "n": // 上边
          newProps.y = originalAnnotation.y + dy;
          newProps.height = originalAnnotation.height - dy;
          break;
        case "e": // 右边
          newProps.width = originalAnnotation.width + dx;
          break;
        case "s": // 下边
          newProps.height = originalAnnotation.height + dy;
          break;
        case "w": // 左边
          newProps.x = originalAnnotation.x + dx;
          newProps.width = originalAnnotation.width - dx;
          break;
      }
    } else if (originalAnnotation.type === "rotated-rectangle") {
      // 旋转框的调整逻辑 - 考虑旋转变换
      const rotationAngle = originalAnnotation.rotation || 0;
      const cos = Math.cos(-rotationAngle); // 逆向旋转
      const sin = Math.sin(-rotationAngle);

      // 将拖拽向量转换到局部坐标系
      const localDx = dx * cos - dy * sin;
      const localDy = dx * sin + dy * cos;

      // 旋转框调整：固定对角点，调整被拖拽的边
      const centerX = originalAnnotation.x + originalAnnotation.width / 2;
      const centerY = originalAnnotation.y + originalAnnotation.height / 2;

      // 计算固定点（对角点）在世界坐标系中的位置
      let fixedLocalX, fixedLocalY;
      let newWidth = originalAnnotation.width;
      let newHeight = originalAnnotation.height;

      switch (resizeHandle.value.handle) {
        case "nw": // 左上角，固定右下角
          fixedLocalX = originalAnnotation.width;
          fixedLocalY = originalAnnotation.height;
          newWidth = Math.max(20, originalAnnotation.width - localDx);
          newHeight = Math.max(20, originalAnnotation.height - localDy);
          break;
        case "ne": // 右上角，固定左下角
          fixedLocalX = 0;
          fixedLocalY = originalAnnotation.height;
          newWidth = Math.max(20, originalAnnotation.width + localDx);
          newHeight = Math.max(20, originalAnnotation.height - localDy);
          break;
        case "se": // 右下角，固定左上角
          fixedLocalX = 0;
          fixedLocalY = 0;
          newWidth = Math.max(20, originalAnnotation.width + localDx);
          newHeight = Math.max(20, originalAnnotation.height + localDy);
          break;
        case "sw": // 左下角，固定右上角
          fixedLocalX = originalAnnotation.width;
          fixedLocalY = 0;
          newWidth = Math.max(20, originalAnnotation.width - localDx);
          newHeight = Math.max(20, originalAnnotation.height + localDy);
          break;
        case "n": // 上边，固定下边
          fixedLocalX = originalAnnotation.width / 2;
          fixedLocalY = originalAnnotation.height;
          newHeight = Math.max(20, originalAnnotation.height - localDy);
          break;
        case "e": // 右边，固定左边
          fixedLocalX = 0;
          fixedLocalY = originalAnnotation.height / 2;
          newWidth = Math.max(20, originalAnnotation.width + localDx);
          break;
        case "s": // 下边，固定上边
          fixedLocalX = originalAnnotation.width / 2;
          fixedLocalY = 0;
          newHeight = Math.max(20, originalAnnotation.height + localDy);
          break;
        case "w": // 左边，固定右边
          fixedLocalX = originalAnnotation.width;
          fixedLocalY = originalAnnotation.height / 2;
          newWidth = Math.max(20, originalAnnotation.width - localDx);
          break;
      }

      // 计算固定点在世界坐标系中的位置
      const cosR = Math.cos(rotationAngle);
      const sinR = Math.sin(rotationAngle);

      const fixedRelativeX = fixedLocalX - originalAnnotation.width / 2;
      const fixedRelativeY = fixedLocalY - originalAnnotation.height / 2;

      const fixedWorldX = fixedRelativeX * cosR - fixedRelativeY * sinR + centerX;
      const fixedWorldY = fixedRelativeX * sinR + fixedRelativeY * cosR + centerY;

      // 计算新的对应点在局部坐标系中的位置
      let newFixedLocalX = fixedLocalX;
      let newFixedLocalY = fixedLocalY;

      // 根据调整的边，计算新的固定点在新矩形中的位置
      switch (resizeHandle.value.handle) {
        case "nw":
          newFixedLocalX = newWidth;
          newFixedLocalY = newHeight;
          break;
        case "ne":
          newFixedLocalX = 0;
          newFixedLocalY = newHeight;
          break;
        case "se":
          newFixedLocalX = 0;
          newFixedLocalY = 0;
          break;
        case "sw":
          newFixedLocalX = newWidth;
          newFixedLocalY = 0;
          break;
        case "n":
          newFixedLocalX = newWidth / 2;
          newFixedLocalY = newHeight;
          break;
        case "e":
          newFixedLocalX = 0;
          newFixedLocalY = newHeight / 2;
          break;
        case "s":
          newFixedLocalX = newWidth / 2;
          newFixedLocalY = 0;
          break;
        case "w":
          newFixedLocalX = newWidth;
          newFixedLocalY = newHeight / 2;
          break;
      }

      // 计算新矩形的中心点，使固定点保持在原位置
      const newFixedRelativeX = newFixedLocalX - newWidth / 2;
      const newFixedRelativeY = newFixedLocalY - newHeight / 2;

      const newCenterX = fixedWorldX - (newFixedRelativeX * cosR - newFixedRelativeY * sinR);
      const newCenterY = fixedWorldY - (newFixedRelativeX * sinR + newFixedRelativeY * cosR);

      // 计算新的左上角位置
      newProps.x = newCenterX - newWidth / 2;
      newProps.y = newCenterY - newHeight / 2;
      newProps.width = newWidth;
      newProps.height = newHeight;
    }

    // 确保宽度和高度为正数
    if (newProps.width > 5 && newProps.height > 5) {
      emit("update-annotation", props.selectedAnnotation.id, newProps);
    }
  } else if (isRotating.value && rotationHandle.value && props.selectedAnnotation) {
    // 旋转标注
    const centerX = props.selectedAnnotation.x + props.selectedAnnotation.width / 2;
    const centerY = props.selectedAnnotation.y + props.selectedAnnotation.height / 2;

    // 计算旋转角度
    const angle = Math.atan2(pos.y - centerY, pos.x - centerX);
    const rotation = angle + Math.PI / 2; // 调整角度偏移

    emit("update-annotation", props.selectedAnnotation.id, {
      rotation: rotation,
    });
  } else if (isDragging.value && props.selectedAnnotation && dragStart.value.annotation) {
    // 拖拽标注 - 基于原始位置计算，避免累加效应
    const dx = pos.x - dragStart.value.x;
    const dy = pos.y - dragStart.value.y;

    emit("update-annotation", props.selectedAnnotation.id, {
      x: dragStart.value.annotation.x + dx,
      y: dragStart.value.annotation.y + dy,
    });
  }
};

const onMouseUp = (event) => {
  if (isDrawing.value && currentAnnotation.value) {
    // 完成标注绘制
    if (currentAnnotation.value.width > 5 && currentAnnotation.value.height > 5) {
      // 发射添加标注事件（不需要在这里设置 id，由 App.vue 统一处理）
      emit("add-annotation", currentAnnotation.value);

      // 画完框后自动切换到选择工具
      emit("switch-tool", "select");

      // 等待标注添加完成后，显示浮动标签选择器
      nextTick(() => {
        // 通过查找最新添加的标注来选中它
        const latestAnnotation = props.annotations[props.annotations.length - 1];
        if (latestAnnotation) {
          emit("select-annotation", latestAnnotation);

          // 计算选择器显示位置（框的右上角）
          const screenPos = canvasToScreen(
            latestAnnotation.x + latestAnnotation.width,
            latestAnnotation.y
          );

          labelSelectorPos.value = {
            x: screenPos.x + 10, // 右边偏移10px
            y: screenPos.y,
          };
          labelSelectorAnnotation.value = latestAnnotation;
          selectedLabel.value = latestAnnotation.label || "";
          labelSelectorShowTime.value = Date.now(); // 记录显示时间
          showLabelSelector.value = true;
        }
      });
    }
    isDrawing.value = false;
    currentAnnotation.value = null;
  }

  // 重置所有拖拽状态
  isDragging.value = false;
  isResizing.value = false;
  isRotating.value = false;
  isPanning.value = false;
  resizeHandle.value = null;
  rotationHandle.value = null;
};

const onWheel = (event) => {
  event.preventDefault();

  const delta = event.deltaY > 0 ? 0.9 : 1.1;
  const newScale = Math.max(0.1, Math.min(5, scale.value * delta));

  const rect = canvas.value.getBoundingClientRect();
  const mouseX = event.clientX - rect.left;
  const mouseY = event.clientY - rect.top;

  // 以鼠标位置为中心缩放
  const scaleDiff = newScale - scale.value;
  offsetX.value -= ((mouseX - offsetX.value) * scaleDiff) / scale.value;
  offsetY.value -= ((mouseY - offsetY.value) * scaleDiff) / scale.value;

  scale.value = newScale;
  draw();
};

// 阻止右键菜单（中键会触发）
const onContextMenu = (event) => {
  event.preventDefault();

  // 右键点击时，检测是否点击了标注
  const rect = canvas.value.getBoundingClientRect();
  const screenPos = {
    x: event.clientX - rect.left,
    y: event.clientY - rect.top,
  };
  const canvasPos = screenToCanvas(event.clientX, event.clientY);

  // 查找点击位置的标注
  const clickedAnnotation = findAnnotationAt(canvasPos);

  if (clickedAnnotation) {
    // 选中该标注
    emit("select-annotation", clickedAnnotation);

    // 显示浮动标签选择器在鼠标位置
    labelSelectorPos.value = {
      x: screenPos.x + 10,
      y: screenPos.y,
    };
    labelSelectorAnnotation.value = clickedAnnotation;
    selectedLabel.value = clickedAnnotation.label || "";
    labelSelectorShowTime.value = Date.now();
    showLabelSelector.value = true;
  }
};

// 检测控制点 (使用屏幕坐标)
const findControlPointAt = (screenPos, annotation) => {
  if (!annotation) return null;

  const tolerance = 15; // 屏幕像素容差，匹配更大的控制点

  if (annotation.type === "rectangle") {
    // 将标注坐标转换为屏幕坐标
    const screenPoints = [
      // 角点
      canvasToScreen(annotation.x, annotation.y),
      canvasToScreen(annotation.x + annotation.width, annotation.y),
      canvasToScreen(annotation.x + annotation.width, annotation.y + annotation.height),
      canvasToScreen(annotation.x, annotation.y + annotation.height),
      // 边中点
      canvasToScreen(annotation.x + annotation.width / 2, annotation.y),
      canvasToScreen(annotation.x + annotation.width, annotation.y + annotation.height / 2),
      canvasToScreen(annotation.x + annotation.width / 2, annotation.y + annotation.height),
      canvasToScreen(annotation.x, annotation.y + annotation.height / 2),
    ];

    const handles = ["nw", "ne", "se", "sw", "n", "e", "s", "w"];
    const types = ["corner", "corner", "corner", "corner", "edge", "edge", "edge", "edge"];

    for (let i = 0; i < screenPoints.length; i++) {
      const point = screenPoints[i];
      const distance = Math.sqrt(
        Math.pow(screenPos.x - point.x, 2) + Math.pow(screenPos.y - point.y, 2)
      );

      if (distance <= tolerance) {
        return {
          x: point.x,
          y: point.y,
          type: types[i],
          handle: handles[i],
        };
      }
    }
  } else if (annotation.type === "rotated-rectangle") {
    // 旋转矩形的控制点检测
    const centerX = annotation.x + annotation.width / 2;
    const centerY = annotation.y + annotation.height / 2;
    const rotation = annotation.rotation || 0;
    const cos = Math.cos(rotation);
    const sin = Math.sin(rotation);

    // 旋转控制点 (在局部坐标系中，与绘制逻辑一致)
    const rotationLocalPoint = { x: annotation.width / 2, y: -25 }; // 局部坐标系中的位置
    // 转换为相对于中心的坐标
    const rotationRelativeX = rotationLocalPoint.x - annotation.width / 2;
    const rotationRelativeY = rotationLocalPoint.y - annotation.height / 2;

    const rotationWorldX = rotationRelativeX * cos - rotationRelativeY * sin + centerX;
    const rotationWorldY = rotationRelativeX * sin + rotationRelativeY * cos + centerY;
    const rotationScreenPoint = canvasToScreen(rotationWorldX, rotationWorldY);

    // 检测旋转控制点
    const rotationDistance = Math.sqrt(
      Math.pow(screenPos.x - rotationScreenPoint.x, 2) +
        Math.pow(screenPos.y - rotationScreenPoint.y, 2)
    );

    if (rotationDistance <= tolerance) {
      return {
        x: rotationScreenPoint.x,
        y: rotationScreenPoint.y,
        type: "rotation",
        handle: "rotate",
      };
    }

    // 角点和边点检测 (在局部坐标系中，与绘制逻辑一致)
    const localCorners = [
      { x: 0, y: 0, handle: "nw" },
      { x: annotation.width, y: 0, handle: "ne" },
      { x: annotation.width, y: annotation.height, handle: "se" },
      { x: 0, y: annotation.height, handle: "sw" },
    ];

    const localEdges = [
      { x: annotation.width / 2, y: 0, handle: "n" },
      { x: annotation.width, y: annotation.height / 2, handle: "e" },
      { x: annotation.width / 2, y: annotation.height, handle: "s" },
      { x: 0, y: annotation.height / 2, handle: "w" },
    ];

    const allLocalPoints = [...localCorners, ...localEdges];
    const handles = ["nw", "ne", "se", "sw", "n", "e", "s", "w"];
    const types = ["corner", "corner", "corner", "corner", "edge", "edge", "edge", "edge"];

    for (let i = 0; i < allLocalPoints.length; i++) {
      const localPoint = allLocalPoints[i];

      // 将局部坐标转换为相对于中心的坐标，然后应用旋转变换
      const relativeX = localPoint.x - annotation.width / 2;
      const relativeY = localPoint.y - annotation.height / 2;

      const worldX = relativeX * cos - relativeY * sin + centerX;
      const worldY = relativeX * sin + relativeY * cos + centerY;

      // 转换到屏幕坐标
      const screenPoint = canvasToScreen(worldX, worldY);

      const distance = Math.sqrt(
        Math.pow(screenPos.x - screenPoint.x, 2) + Math.pow(screenPos.y - screenPoint.y, 2)
      );

      if (distance <= tolerance) {
        return {
          x: screenPoint.x,
          y: screenPoint.y,
          type: types[i],
          handle: handles[i],
        };
      }
    }
  }

  return null;
};

// 鼠标按下事件处理
const onMouseDown = (event) => {
  event.preventDefault();

  // 如果没有图片，不允许绘制
  if (
    !props.image &&
    (props.currentTool === "rectangle" || props.currentTool === "rotated-rectangle")
  ) {
    return;
  }

  const rect = canvas.value.getBoundingClientRect();
  const screenPos = {
    x: event.clientX - rect.left,
    y: event.clientY - rect.top,
  };
  const canvasPos = screenToCanvas(event.clientX, event.clientY);
  startPoint.value = canvasPos;

  // 只处理左键点击
  if (event.button !== 0) {
    return;
  }

  // 通用交互检测：不论当前工具是什么，都检查是否可以进行交互

  // 首先检查是否点击了选中标注的控制点
  if (props.selectedAnnotation) {
    const controlPoint = findControlPointAt(screenPos, props.selectedAnnotation);
    if (controlPoint) {
      if (controlPoint.type === "rotation") {
        isRotating.value = true;
        rotationHandle.value = controlPoint;
      } else {
        isResizing.value = true;
        resizeHandle.value = controlPoint;
      }
      dragStart.value = {
        x: canvasPos.x,
        y: canvasPos.y,
        annotation: { ...props.selectedAnnotation },
      };
      return;
    }
  }

  // 检查是否点击了任意标注的控制点（即使未选中）
  for (const annotation of props.annotations) {
    if (annotation.id !== props.selectedAnnotation?.id) {
      const controlPoint = findControlPointAt(screenPos, annotation);
      if (controlPoint) {
        // 自动选中该标注并开始交互
        emit("select-annotation", annotation);

        if (controlPoint.type === "rotation") {
          isRotating.value = true;
          rotationHandle.value = controlPoint;
        } else {
          isResizing.value = true;
          resizeHandle.value = controlPoint;
        }
        dragStart.value = {
          x: canvasPos.x,
          y: canvasPos.y,
          annotation: { ...annotation },
        };
        return;
      }
    }
  }

  if (props.currentTool === "select") {
    // 检查是否点击了标注内部（用于移动）
    const clickedAnnotation = findAnnotationAt(canvasPos);
    if (clickedAnnotation) {
      emit("select-annotation", clickedAnnotation);
      isDragging.value = true;
      dragStart.value = {
        x: canvasPos.x,
        y: canvasPos.y,
        annotation: { ...clickedAnnotation }, // 保存原始标注位置
        offsetX: offsetX.value,
        offsetY: offsetY.value,
      };
    } else {
      // 点击空白处：开始拖动画布
      emit("select-annotation", null);
      isPanning.value = true;
      dragStart.value = {
        x: event.clientX,
        y: event.clientY,
        offsetX: offsetX.value,
        offsetY: offsetY.value,
      };
    }
  } else if (props.currentTool === "rectangle") {
    // 开始绘制矩形
    isDrawing.value = true;
    currentAnnotation.value = {
      type: "rectangle",
      x: canvasPos.x,
      y: canvasPos.y,
      width: 0,
      height: 0,
      label: "",
    };
  } else if (props.currentTool === "rotated-rectangle") {
    // 开始绘制旋转矩形
    isDrawing.value = true;
    currentAnnotation.value = {
      type: "rotated-rectangle",
      x: canvasPos.x,
      y: canvasPos.y,
      width: 0,
      height: 0,
      rotation: 0,
      label: "",
    };
  }
};

const findAnnotationAt = (pos) => {
  // 从后往前检查，优先选择最上层的标注
  for (let i = props.annotations.length - 1; i >= 0; i--) {
    const annotation = props.annotations[i];

    if (annotation.type === "rectangle") {
      if (
        pos.x >= annotation.x &&
        pos.x <= annotation.x + annotation.width &&
        pos.y >= annotation.y &&
        pos.y <= annotation.y + annotation.height
      ) {
        return annotation;
      }
    } else if (annotation.type === "rotated-rectangle") {
      // 旋转矩形的点击检测
      const centerX = annotation.x + annotation.width / 2;
      const centerY = annotation.y + annotation.height / 2;
      const rotation = annotation.rotation || 0;

      // 将点击位置转换到局部坐标系（逆向旋转）
      const cos = Math.cos(-rotation);
      const sin = Math.sin(-rotation);

      const localX = (pos.x - centerX) * cos - (pos.y - centerY) * sin + annotation.width / 2;
      const localY = (pos.x - centerX) * sin + (pos.y - centerY) * cos + annotation.height / 2;

      // 在局部坐标系中检查是否在矩形内
      if (localX >= 0 && localX <= annotation.width && localY >= 0 && localY <= annotation.height) {
        return annotation;
      }
    }
  }
  return null;
};

// 更新鼠标光标样式
const updateCursor = (event) => {
  if (!canvas.value) return;

  let cursor = "default";

  // 如果正在拖动画布，显示抓取光标
  if (isPanning.value) {
    cursor = "grabbing";
    canvas.value.style.cursor = cursor;
    return;
  }

  const rect = canvas.value.getBoundingClientRect();
  const screenPos = {
    x: event.clientX - rect.left,
    y: event.clientY - rect.top,
  };
  const canvasPos = screenToCanvas(event.clientX, event.clientY);

  // 优先检查所有标注的控制点（不论是否选中）
  let foundControlPoint = false;

  // 检查选中标注的控制点
  if (props.selectedAnnotation) {
    const controlPoint = findControlPointAt(screenPos, props.selectedAnnotation);
    if (controlPoint) {
      foundControlPoint = true;
      switch (controlPoint.handle) {
        case "nw":
        case "se":
          cursor = "nw-resize";
          break;
        case "ne":
        case "sw":
          cursor = "ne-resize";
          break;
        case "n":
        case "s":
          cursor = "n-resize";
          break;
        case "e":
        case "w":
          cursor = "e-resize";
          break;
        case "rotate":
          cursor = "grab";
          break;
      }
    }
  }

  // 如果没有找到选中标注的控制点，检查其他标注的控制点
  if (!foundControlPoint) {
    for (const annotation of props.annotations) {
      if (annotation.id !== props.selectedAnnotation?.id) {
        const controlPoint = findControlPointAt(screenPos, annotation);
        if (controlPoint) {
          foundControlPoint = true;
          switch (controlPoint.handle) {
            case "nw":
            case "se":
              cursor = "nw-resize";
              break;
            case "ne":
            case "sw":
              cursor = "ne-resize";
              break;
            case "n":
            case "s":
              cursor = "n-resize";
              break;
            case "e":
            case "w":
              cursor = "e-resize";
              break;
            case "rotate":
              cursor = "grab";
              break;
          }
          break;
        }
      }
    }
  }

  // 如果没有控制点，检查是否在标注内部
  if (!foundControlPoint) {
    const annotation = findAnnotationAt(canvasPos);
    if (annotation) {
      cursor = "move";
    } else if (props.currentTool === "rectangle" || props.currentTool === "rotated-rectangle") {
      cursor = "crosshair";
    }
  }

  canvas.value.style.cursor = cursor;
};

// 更新悬停状态
const updateHoverState = (event) => {
  const rect = canvas.value.getBoundingClientRect();
  const screenPos = {
    x: event.clientX - rect.left,
    y: event.clientY - rect.top,
  };
  const canvasPos = screenToCanvas(event.clientX, event.clientY);

  let newHoveredAnnotation = null;

  // 检查是否悬停在任何标注的控制点上
  for (const annotation of props.annotations) {
    const controlPoint = findControlPointAt(screenPos, annotation);
    if (controlPoint) {
      newHoveredAnnotation = annotation;
      break;
    }
  }

  // 如果没有悬停在控制点上，检查是否悬停在标注内部
  if (!newHoveredAnnotation) {
    newHoveredAnnotation = findAnnotationAt(canvasPos);
  }

  // 只有悬停状态改变时才重绘
  if (hoveredAnnotation.value?.id !== newHoveredAnnotation?.id) {
    hoveredAnnotation.value = newHoveredAnnotation;
    draw(); // 重绘以显示/隐藏控制点
  }
};

// 图片加载
const loadImage = () => {
  if (!props.image) return;

  imageElement.value = new Image();
  imageElement.value.onload = () => {
    fitImageToCanvas();
    draw();
  };
  imageElement.value.src = props.image.url || props.image.data;
};

const fitImageToCanvas = () => {
  if (!imageElement.value || !props.image) return;

  // 使用原始图片尺寸来计算缩放（而非预览图尺寸）
  const imgWidth = props.image.width;
  const imgHeight = props.image.height;

  const scaleX = containerWidth.value / imgWidth;
  const scaleY = containerHeight.value / imgHeight;
  scale.value = Math.min(scaleX, scaleY) * 0.9;

  offsetX.value = (containerWidth.value - imgWidth * scale.value) / 2;
  offsetY.value = (containerHeight.value - imgHeight * scale.value) / 2;
};

// 标签选择处理
const onLabelSelect = (value) => {
  if (labelSelectorAnnotation.value) {
    const trimmedValue = value ? value.trim() : "";

    if (trimmedValue) {
      emit("update-annotation", labelSelectorAnnotation.value.id, {
        label: trimmedValue,
      });
      closeLabelSelector();
    }
  }
};

const closeLabelSelector = () => {
  showLabelSelector.value = false;
  labelSelectorAnnotation.value = null;
  selectedLabel.value = "";
};

// 记录浮动框最近一次显示的时间
const labelSelectorShowTime = ref(0);

// 点击画布其他地方关闭标签选择器
const onCanvasClick = (event) => {
  // 如果浮动框刚显示（100ms内），不立即关闭（避免刚画完就关闭）
  if (Date.now() - labelSelectorShowTime.value < 100) {
    return;
  }

  // 如果点击的不是标签选择器，关闭它
  if (showLabelSelector.value) {
    const target = event.target;
    // 检查是否点击了标签选择器或其子元素
    const labelSelector = document.querySelector(".label-selector");
    if (labelSelector && !labelSelector.contains(target)) {
      closeLabelSelector();
    }
  }
};

// 监听器
watch(() => props.image, loadImage, { immediate: true });
watch(() => props.annotations, draw, { deep: true });
watch(() => props.selectedAnnotation, draw);
watch(() => props.showAnnotations, draw); // 监听显示/隐藏状态

// 键盘事件处理
const onKeyDown = (event) => {
  if (event.key === "Escape" && showLabelSelector.value) {
    event.preventDefault();
    closeLabelSelector();
  }
};

const onKeyUp = (event) => {
  // 预留用于将来的键盘事件处理
};

// 生命周期
onMounted(() => {
  nextTick(() => {
    initCanvas();

    // 添加事件监听器
    window.addEventListener("resize", initCanvas);
    window.addEventListener("keydown", onKeyDown);
    window.addEventListener("keyup", onKeyUp);
  });
});

onUnmounted(() => {
  window.removeEventListener("resize", initCanvas);
  window.removeEventListener("keydown", onKeyDown);
  window.removeEventListener("keyup", onKeyUp);
});
</script>

<template>
  <div ref="canvasContainer" class="canvas-container">
    <canvas
      ref="canvas"
      @mousedown="onMouseDown"
      @mousemove="onMouseMove"
      @mouseup="onMouseUp"
      @wheel="onWheel"
      @contextmenu="onContextMenu"
      @click="onCanvasClick"
      class="annotation-canvas"
    />

    <div v-if="!image" class="empty-canvas">
      <n-icon size="48">
        <svg viewBox="0 0 24 24">
          <path
            fill="currentColor"
            d="M21 19V5c0-1.1-.9-2-2-2H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2zM8.5 13.5l2.5 3.01L14.5 12l4.5 6H5l3.5-4.5z"
          />
        </svg>
      </n-icon>
      <p>请选择图片文件</p>
    </div>

    <div class="canvas-info">
      <span>缩放: {{ Math.round(scale * 100) }}%</span>
      <span v-if="image"> {{ Math.round(mousePos.x) }}, {{ Math.round(mousePos.y) }} </span>
    </div>

    <!-- 浮动标签选择器 -->
    <div
      v-if="showLabelSelector"
      class="label-selector"
      :style="{
        left: labelSelectorPos.x + 'px',
        top: labelSelectorPos.y + 'px',
      }"
    >
      <n-auto-complete
        v-model:value="selectedLabel"
        :options="labelOptions"
        placeholder="输入或选择标签"
        size="small"
        clearable
        :get-show="() => labelOptions.length > 0"
        @select="onLabelSelect"
      >
        <template #default="{ handleInput, handleBlur, handleFocus, value }">
          <n-input
            :value="value"
            placeholder="输入或选择标签"
            size="small"
            autofocus
            @input="handleInput"
            @focus="handleFocus"
            @blur="handleBlur"
            @keydown.enter.prevent="onLabelSelect(selectedLabel)"
          />
        </template>
      </n-auto-complete>
      <n-button size="small" text @click="closeLabelSelector" style="margin-left: 4px">
        <n-icon size="14">
          <svg viewBox="0 0 24 24">
            <path
              fill="currentColor"
              d="M19,6.41L17.59,5L12,10.59L6.41,5L5,6.41L10.59,12L5,17.59L6.41,19L12,13.41L17.59,19L19,17.59L13.41,12L19,6.41Z"
            />
          </svg>
        </n-icon>
      </n-button>
    </div>
  </div>
</template>

<style scoped>
.canvas-container {
  position: relative;
  width: 100%;
  height: 100%;
  overflow: hidden;
  background-color: #f5f5f5;
}

.annotation-canvas {
  cursor: crosshair;
  display: block;
}

.annotation-canvas.select-mode {
  cursor: default;
}

.empty-canvas {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  text-align: center;
  color: #999;
}

.empty-canvas p {
  margin: 12px 0 0 0;
  font-size: 14px;
}

.canvas-info {
  position: absolute;
  bottom: 8px;
  left: 8px;
  background-color: rgba(0, 0, 0, 0.7);
  color: white;
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 11px;
  display: flex;
  gap: 12px;
}

.canvas-hint {
  position: absolute;
  top: 8px;
  left: 8px;
  background-color: rgba(33, 150, 243, 0.9);
  color: white;
  padding: 6px 12px;
  border-radius: 4px;
  font-size: 12px;
  animation: fadeIn 0.5s ease-in;
  pointer-events: none;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(-10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* 强制浅色主题样式 */
.canvas-container {
  background-color: #f5f5f5 !important;
}

.empty-canvas {
  color: #cccccc !important;
}

.empty-canvas p {
  color: #cccccc !important;
}

.canvas-info {
  background-color: rgba(0, 0, 0, 0.8) !important;
  color: #ffffff !important;
}

/* 浮动标签选择器 */
.label-selector {
  position: absolute;
  z-index: 1000;
  background-color: #ffffff;
  padding: 8px;
  border-radius: 6px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  display: flex;
  align-items: center;
  gap: 4px;
  animation: fadeInScale 0.2s ease-out;
}

@keyframes fadeInScale {
  from {
    opacity: 0;
    transform: scale(0.9);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}

.label-selector :deep(.n-auto-complete) {
  width: 200px;
}
</style>
