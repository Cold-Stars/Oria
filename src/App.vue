<script setup>
import { ref, onMounted, onUnmounted, h } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { NConfigProvider, createDiscreteApi } from "naive-ui";
import ToolBar from "./components/ToolBar.vue";
import FileList from "./components/FileList.vue";
import AnnotationCanvas from "./components/AnnotationCanvas.vue";
import PropertyPanel from "./components/PropertyPanel.vue";
import KeyboardHelp from "./components/KeyboardHelp.vue";
import InferenceSettings from "./components/InferenceSettings.vue";
// import SimpleCanvas from "./components/SimpleCanvas.vue"; // 测试完成，已移除
import { createKeyboardManager } from "./utils/keyboard.js";
import { isImageFile, validateAnnotation } from "./utils/annotation.js";

// 创建独立的 Naive UI API（不需要 provider）
const configProviderPropsRef = ref({
  messageProviderProps: {
    placement: "top",
  },
});

const { message, dialog } = createDiscreteApi(["message", "dialog"], {
  configProviderProps: configProviderPropsRef,
  messageProviderProps: {
    placement: "top",
    containerStyle: {
      top: "120px",
    },
  },
});

// 应用状态
const currentTool = ref("select"); // select, rectangle, rotated-rectangle
const currentImage = ref(null);
const annotations = ref([]);
const selectedAnnotation = ref(null);
const imageFiles = ref([]);
const currentImageIndex = ref(-1);
const isLoading = ref(false);
const errorMessage = ref("");
const keyboardManager = ref(null);
const showKeyboardHelp = ref(false);
const showAnnotations = ref(true); // 是否显示标注框
const showInferenceSettings = ref(false); // 是否显示推理设置对话框
const inferenceConfig = ref(null); // 推理配置

// 保存状态
const saveStatus = ref("saved"); // saved, saving, unsaved, error
const lastSaveTime = ref(null);
const autoSaveTimer = ref(null);
const hasUnsavedChanges = ref(false);

// 记录最近使用的绘制工具
const lastDrawingTool = ref("rectangle"); // 默认为矩形框

// 标签类别管理
const labelCategories = ref([]); // 所有出现过的标签类别
const categoryColors = ref({}); // 类别对应的颜色映射
const lastUsedLabel = ref(""); // 最近使用的标签

// 生成类别颜色的函数（基于字符串哈希）
const generateCategoryColor = (category) => {
  // 使用类别名称生成稳定的哈希值
  let hash = 0;
  for (let i = 0; i < category.length; i++) {
    hash = category.charCodeAt(i) + ((hash << 5) - hash);
    hash = hash & hash; // 转换为32位整数
  }

  // 使用哈希值生成色相（0-360度）
  const hue = Math.abs(hash % 360);

  // 使用HSL色彩空间生成颜色
  return `hsl(${hue}, 70%, 50%)`; // 饱和度70%，亮度50%
};

// 从标注中提取所有类别
const extractCategories = () => {
  const categoriesSet = new Set(labelCategories.value); // 保留现有类别

  // 遍历当前图片的所有标注，收集新类别
  annotations.value.forEach((annotation) => {
    if (annotation.label && annotation.label.trim()) {
      categoriesSet.add(annotation.label.trim());
    }
  });

  // 更新类别列表（合并模式，不会丢失之前的类别）
  const allCategories = Array.from(categoriesSet).sort();
  const addedCount = allCategories.length - labelCategories.value.length;
  labelCategories.value = allCategories;

  // 为新类别分配颜色（基于类别名称哈希，保证稳定）
  allCategories.forEach((category) => {
    if (!categoryColors.value[category]) {
      categoryColors.value[category] = generateCategoryColor(category);
    }
  });
};

// 添加新类别
const addCategory = (category) => {
  if (!category || !category.trim()) {
    return;
  }

  const trimmedCategory = category.trim();

  if (!labelCategories.value.includes(trimmedCategory)) {
    labelCategories.value.push(trimmedCategory);
    labelCategories.value.sort(); // 保持排序

    // 分配颜色（基于类别名称哈希，保证稳定且唯一）
    categoryColors.value[trimmedCategory] = generateCategoryColor(trimmedCategory);
  }
};

// 获取类别颜色
const getCategoryColor = (category) => {
  if (!category) return "#00aaff"; // 默认蓝色
  return categoryColors.value[category] || "#00aaff";
};

// 工具切换
const switchTool = (tool) => {
  currentTool.value = tool;

  // 记录最近使用的绘制工具
  if (tool === "rectangle" || tool === "rotated-rectangle") {
    lastDrawingTool.value = tool;
  }
};

// 切换标注显示/隐藏
const toggleAnnotations = () => {
  showAnnotations.value = !showAnnotations.value;
};

// 导出/导入功能
const exportToCoco = async () => {
  if (imageFiles.value.length === 0) {
    message.warning("请先打开文件夹");
    return;
  }

  if (labelCategories.value.length === 0) {
    message.warning("没有标签类别，无法导出");
    return;
  }

  try {
    isLoading.value = true;
    const imagePath = imageFiles.value[0];

    // 获取文件夹路径（处理Windows路径）
    const imageFolder = imagePath.substring(0, imagePath.lastIndexOf("\\"));

    // 让用户选择保存文件夹
    let savePath;
    try {
      savePath = await invoke("select_save_folder");
    } catch (error) {
      // 用户取消选择文件夹，静默处理
      const errorMsg = typeof error === "string" ? error : error.message || JSON.stringify(error);
      if (errorMsg.includes("No folder selected") || errorMsg.includes("未选择")) {
        return;
      }
      throw error; // 其他错误继续抛出
    }

    if (!savePath) {
      isLoading.value = false;
      return;
    }

    // 导出COCO格式
    const result = await invoke("export_coco", {
      imageFolder: imageFolder, // 图片所在文件夹
      exportPath: savePath, // 用户选择的保存路径
    });

    message.success("COCO格式导出成功！", {
      duration: 3000,
    });
    dialog.info({
      title: "导出成功",
      content: `文件位置: ${result}`,
      positiveText: "确定",
    });
  } catch (error) {
    const errorMsg = typeof error === "string" ? error : error.message || JSON.stringify(error);
    console.error("导出COCO失败:", error);
    message.error(`导出COCO失败: ${errorMsg}`);
  } finally {
    isLoading.value = false;
  }
};

const exportToYolo = async () => {
  if (imageFiles.value.length === 0) {
    message.warning("请先打开文件夹");
    return;
  }

  if (labelCategories.value.length === 0) {
    message.warning("没有标签类别，无法导出");
    return;
  }

  try {
    isLoading.value = true;
    const imagePath = imageFiles.value[0];
    const imageFolder = imagePath.substring(0, imagePath.lastIndexOf("\\"));

    // 构建类别映射
    const classMap = {};
    labelCategories.value.forEach((category, index) => {
      classMap[category] = index;
    });

    // 让用户选择保存文件夹
    let savePath;
    try {
      savePath = await invoke("select_save_folder");
    } catch (error) {
      // 用户取消选择文件夹，静默处理
      const errorMsg = typeof error === "string" ? error : error.message || JSON.stringify(error);
      if (errorMsg.includes("No folder selected") || errorMsg.includes("未选择")) {
        return;
      }
      throw error; // 其他错误继续抛出
    }

    if (!savePath) {
      isLoading.value = false;
      return;
    }

    // 检查是否有旋转框
    const hasRotatedBox = annotations.value.some((ann) => ann.type === "rotated-rectangle");

    let forceRectangle = false;

    // 如果有旋转框，让用户选择导出类型
    if (hasRotatedBox) {
      await new Promise((resolve) => {
        dialog.warning({
          title: "选择导出类型",
          content: "检测到旋转框标注，请选择导出格式：",
          positiveText: "导出为旋转框（OBB）",
          negativeText: "导出为矩形框",
          onPositiveClick: () => {
            forceRectangle = false;
            resolve();
          },
          onNegativeClick: () => {
            forceRectangle = true;
            resolve();
          },
        });
      });
    }

    // 导出YOLO格式
    const result = await invoke("export_yolo_batch", {
      imageFolder: imageFolder, // 图片所在文件夹
      config: {
        class_map: classMap, // 类别映射
        export_path: savePath, // 用户选择的保存路径
        force_rectangle: forceRectangle, // 是否强制导出为矩形框
      },
    });

    message.success("YOLOv8格式导出成功！", {
      duration: 3000,
    });
    dialog.info({
      title: "导出成功",
      content: `导出了 ${result.length} 个文件`,
      positiveText: "确定",
    });
  } catch (error) {
    const errorMsg = typeof error === "string" ? error : error.message || JSON.stringify(error);
    console.error("导出YOLO失败:", error);
    message.error(`导出YOLO失败: ${errorMsg}`);
  } finally {
    isLoading.value = false;
  }
};

const importFromCoco = async () => {
  if (imageFiles.value.length === 0) {
    message.warning("请先打开图片所在的文件夹");
    return;
  }

  try {
    isLoading.value = true;
    const imagePath = imageFiles.value[0];
    const folderPath = imagePath.substring(0, imagePath.lastIndexOf("\\"));

    // 首先选择COCO JSON文件
    const cocoFilePath = await invoke("select_coco_file");

    if (!cocoFilePath) {
      isLoading.value = false;
      return;
    }

    // 让用户选择导入类型
    const importTypeRef = ref("auto"); // 默认自动判断
    let dialogCancelled = false;

    await new Promise((resolve) => {
      const d = dialog.create({
        title: "选择导入类型",
        content: () => {
          return h("div", { style: { padding: "16px 0" } }, [
            h(
              "p",
              { style: { marginBottom: "16px", color: "#666" } },
              "请选择如何导入COCO标注数据："
            ),
            h(
              "div",
              {
                class: "n-radio-group",
                style: { display: "flex", flexDirection: "column", gap: "12px" },
              },
              [
                h(
                  "label",
                  {
                    class: "radio-item",
                    style: {
                      display: "flex",
                      alignItems: "center",
                      padding: "8px 12px",
                      border: "1px solid #e0e0e0",
                      borderRadius: "4px",
                      cursor: "pointer",
                      transition: "all 0.2s",
                      backgroundColor: importTypeRef.value === "auto" ? "#f0f9ff" : "#fff",
                    },
                    onClick: () => {
                      importTypeRef.value = "auto";
                    },
                  },
                  [
                    h("input", {
                      type: "radio",
                      name: "importType",
                      value: "auto",
                      checked: importTypeRef.value === "auto",
                      style: { marginRight: "8px" },
                      onChange: () => {
                        importTypeRef.value = "auto";
                      },
                    }),
                    h("div", [
                      h(
                        "div",
                        { style: { fontWeight: "500", marginBottom: "4px" } },
                        "🔍 自动判断（推荐）"
                      ),
                      h(
                        "div",
                        { style: { fontSize: "12px", color: "#999" } },
                        "根据数据中的旋转信息自动识别类型"
                      ),
                    ]),
                  ]
                ),
                h(
                  "label",
                  {
                    class: "radio-item",
                    style: {
                      display: "flex",
                      alignItems: "center",
                      padding: "8px 12px",
                      border: "1px solid #e0e0e0",
                      borderRadius: "4px",
                      cursor: "pointer",
                      transition: "all 0.2s",
                      backgroundColor: importTypeRef.value === "rectangle" ? "#f0f9ff" : "#fff",
                    },
                    onClick: () => {
                      importTypeRef.value = "rectangle";
                    },
                  },
                  [
                    h("input", {
                      type: "radio",
                      name: "importType",
                      value: "rectangle",
                      checked: importTypeRef.value === "rectangle",
                      style: { marginRight: "8px" },
                      onChange: () => {
                        importTypeRef.value = "rectangle";
                      },
                    }),
                    h("div", [
                      h(
                        "div",
                        { style: { fontWeight: "500", marginBottom: "4px" } },
                        "▭ 强制矩形框"
                      ),
                      h(
                        "div",
                        { style: { fontSize: "12px", color: "#999" } },
                        "全部导入为普通矩形框，忽略旋转信息"
                      ),
                    ]),
                  ]
                ),
                h(
                  "label",
                  {
                    class: "radio-item",
                    style: {
                      display: "flex",
                      alignItems: "center",
                      padding: "8px 12px",
                      border: "1px solid #e0e0e0",
                      borderRadius: "4px",
                      cursor: "pointer",
                      transition: "all 0.2s",
                      backgroundColor:
                        importTypeRef.value === "rotated-rectangle" ? "#f0f9ff" : "#fff",
                    },
                    onClick: () => {
                      importTypeRef.value = "rotated-rectangle";
                    },
                  },
                  [
                    h("input", {
                      type: "radio",
                      name: "importType",
                      value: "rotated-rectangle",
                      checked: importTypeRef.value === "rotated-rectangle",
                      style: { marginRight: "8px" },
                      onChange: () => {
                        importTypeRef.value = "rotated-rectangle";
                      },
                    }),
                    h("div", [
                      h(
                        "div",
                        { style: { fontWeight: "500", marginBottom: "4px" } },
                        "◇ 强制旋转框"
                      ),
                      h(
                        "div",
                        { style: { fontSize: "12px", color: "#999" } },
                        "全部导入为旋转框，未旋转的设为0°"
                      ),
                    ]),
                  ]
                ),
              ]
            ),
          ]);
        },
        positiveText: "确定导入",
        negativeText: "取消",
        onPositiveClick: () => {
          d.destroy();
          resolve();
        },
        onNegativeClick: () => {
          dialogCancelled = true;
          d.destroy();
          resolve();
        },
      });
    });

    // 如果用户取消，直接返回
    if (dialogCancelled) {
      isLoading.value = false;
      return;
    }

    // 导入COCO格式
    const result = await invoke("import_coco", {
      cocoFilePath: cocoFilePath,
      imageFolder: folderPath,
      importType: importTypeRef.value,
    });

    message.success("COCO格式导入成功！", {
      duration: 3000,
    });

    dialog.info({
      title: "导入成功",
      content: result,
      positiveText: "确定",
    });

    // 刷新当前图片以显示导入的标注
    if (currentImageIndex.value >= 0) {
      await loadImage(currentImageIndex.value);
    }
  } catch (error) {
    // 如果用户取消选择文件，不显示错误提示
    const errorMsg = typeof error === "string" ? error : error.message || JSON.stringify(error);
    if (errorMsg.includes("未选择文件") || errorMsg.includes("No file selected")) {
      // 用户取消操作，静默处理
      return;
    } else {
      console.error("导入COCO失败:", error);
      message.error(`导入COCO失败: ${errorMsg}`);
    }
  } finally {
    isLoading.value = false;
  }
};

// 文件操作
const openFolder = async () => {
  try {
    isLoading.value = true;
    errorMessage.value = "";
    const files = await invoke("open_folder");
    // 过滤出图片文件
    const imageFilesList = files.filter((file) => isImageFile(file));
    imageFiles.value = imageFilesList;

    // 打开新文件夹时，清空类别列表（会在加载图片时重新提取）
    labelCategories.value = [];
    categoryColors.value = {};

    if (imageFilesList.length > 0) {
      loadImage(0);
    } else {
      errorMessage.value = "所选文件夹中没有找到支持的图片文件";
    }
  } catch (error) {
    // 如果用户取消选择，不显示错误提示
    const errorMsg = typeof error === "string" ? error : error.message || JSON.stringify(error);
    if (errorMsg.includes("No folder selected") || errorMsg.includes("未选择")) {
      // 用户取消操作，静默处理
      return;
    } else {
      console.error("打开文件夹失败:", error);
      errorMessage.value = "打开文件夹失败: " + errorMsg;
    }
  } finally {
    isLoading.value = false;
  }
};

const loadImage = async (index) => {
  if (index >= 0 && index < imageFiles.value.length) {
    // 切换图片前，如果有未保存的更改，先保存
    if (hasUnsavedChanges.value && currentImageIndex.value >= 0) {
      await saveAnnotations();
    }

    // 清除自动保存定时器
    if (autoSaveTimer.value) {
      clearTimeout(autoSaveTimer.value);
      autoSaveTimer.value = null;
    }

    currentImageIndex.value = index;
    try {
      // 使用带缓存的图片加载
      const imageData = await invoke("load_image_cached", {
        path: imageFiles.value[index],
      });
      currentImage.value = imageData;

      // 加载对应的标注数据
      await loadAnnotations();

      // 重置保存状态
      saveStatus.value = "saved";
      hasUnsavedChanges.value = false;

      // 预加载前后图片（异步，不阻塞）
      preloadNearbyImages(index);
    } catch (error) {
      console.error("Failed to load image:", error);
    }
  }
};

// 预加载当前图片前后的图片
const preloadNearbyImages = async (currentIndex) => {
  try {
    await invoke("preload_images", {
      currentIndex,
      allPaths: imageFiles.value,
      preloadCount: 3, // 前后各预加载3张（加大预加载范围）
    });
  } catch (error) {
    console.error("Failed to preload images:", error);
  }
};

const loadAnnotations = async () => {
  if (currentImageIndex.value >= 0) {
    try {
      const annotationData = await invoke("load_annotations", {
        imagePath: imageFiles.value[currentImageIndex.value],
      });
      annotations.value = annotationData || [];

      // 提取类别
      extractCategories();
    } catch (error) {
      console.error("Failed to load annotations:", error);
      annotations.value = [];
    }
  }
};

const saveAnnotations = async () => {
  if (currentImageIndex.value >= 0 && imageFiles.value[currentImageIndex.value]) {
    try {
      saveStatus.value = "saving";

      await invoke("save_annotations", {
        imagePath: imageFiles.value[currentImageIndex.value],
        annotations: annotations.value,
      });

      saveStatus.value = "saved";
      lastSaveTime.value = new Date();
      hasUnsavedChanges.value = false;
    } catch (error) {
      console.error("保存标注失败:", error);
      saveStatus.value = "error";
      errorMessage.value = "保存失败: " + error;

      // 3秒后清除错误状态
      setTimeout(() => {
        if (saveStatus.value === "error") {
          saveStatus.value = hasUnsavedChanges.value ? "unsaved" : "saved";
        }
      }, 3000);
    }
  }
};

// 自动保存函数
const autoSave = () => {
  // 清除之前的定时器
  if (autoSaveTimer.value) {
    clearTimeout(autoSaveTimer.value);
  }

  // 标记为未保存
  hasUnsavedChanges.value = true;
  saveStatus.value = "unsaved";

  // 延迟2秒后自动保存
  autoSaveTimer.value = setTimeout(() => {
    saveAnnotations();
  }, 2000); // 2秒延迟，避免频繁保存
};

// 标注操作
const addAnnotation = (annotation) => {
  const newAnnotation = {
    id: Date.now().toString(), // 转换为字符串
    created: new Date().toISOString(), // 添加创建时间
    visible: true, // 默认可见
    ...annotation,
  };

  // 如果没有标签，使用上次使用的标签
  if (!newAnnotation.label && lastUsedLabel.value) {
    newAnnotation.label = lastUsedLabel.value;
  }

  annotations.value.push(newAnnotation);
  selectedAnnotation.value = newAnnotation; // 选中新创建的标注对象

  // 如果标注有类别，添加到类别列表
  if (newAnnotation.label && newAnnotation.label.trim()) {
    addCategory(newAnnotation.label.trim());
    lastUsedLabel.value = newAnnotation.label.trim();
  }

  autoSave(); // 触发自动保存
};

const updateAnnotation = (id, updates) => {
  const index = annotations.value.findIndex((ann) => ann.id === id);

  if (index !== -1) {
    // 使用响应式更新
    annotations.value[index] = { ...annotations.value[index], ...updates };

    // 如果当前选中的是这个标注，同步更新选中的引用
    if (selectedAnnotation.value && selectedAnnotation.value.id === id) {
      selectedAnnotation.value = annotations.value[index];
    }

    // 如果更新了标签，检查是否需要添加新类别并记录
    if (updates.label !== undefined) {
      if (updates.label && updates.label.trim()) {
        addCategory(updates.label.trim());
        lastUsedLabel.value = updates.label.trim();
      }
    }

    autoSave(); // 触发自动保存
  }
};

const deleteAnnotation = (id) => {
  annotations.value = annotations.value.filter((ann) => ann.id !== id);
  selectedAnnotation.value = null;
  autoSave(); // 触发自动保存
};

const selectAnnotation = (annotation) => {
  selectedAnnotation.value = annotation;
};

// 导航功能
const nextImage = () => {
  if (currentImageIndex.value < imageFiles.value.length - 1) {
    loadImage(currentImageIndex.value + 1);
  }
};

const prevImage = () => {
  if (currentImageIndex.value > 0) {
    loadImage(currentImageIndex.value - 1);
  }
};

const firstImage = () => {
  if (imageFiles.value.length > 0) {
    loadImage(0);
  }
};

const lastImage = () => {
  if (imageFiles.value.length > 0) {
    loadImage(imageFiles.value.length - 1);
  }
};

// 工具快捷键
const selectTool = () => switchTool("select");
const rectangleTool = () => {
  if (!currentImage.value) return;
  switchTool("rectangle");
};
const rotatedRectangleTool = () => {
  if (!currentImage.value) return;
  switchTool("rotated-rectangle");
};

// 快速创建标注（使用最近的工具）
const createAnnotation = () => {
  // 如果没有图片，不允许创建标注
  if (!currentImage.value) {
    return;
  }
  switchTool(lastDrawingTool.value);
};

// 标注操作快捷键
const deleteSelectedAnnotation = () => {
  if (selectedAnnotation.value) {
    deleteAnnotation(selectedAnnotation.value.id);
  }
};

const clearSelection = () => {
  selectedAnnotation.value = null;
};

// 显示帮助
const toggleKeyboardHelp = () => {
  showKeyboardHelp.value = !showKeyboardHelp.value;
};

// 显示自动标注对话框
// 显示推理设置对话框
const showInferenceSettingsDialog = () => {
  showInferenceSettings.value = true;
};

// 保存推理设置
const handleSettingsSaved = (settings) => {
  inferenceConfig.value = settings;
  message.success("推理配置已更新");
};

// 推理当前图片
const inferenceOne = async () => {
  if (!inferenceConfig.value) {
    message.warning("请先配置推理参数");
    showInferenceSettings.value = true;
    return;
  }
  if (currentImageIndex.value < 0) {
    message.warning("请先选择图片");
    return;
  }

  await runInference(currentImageIndex.value, 1);
};

// 推理当前及之后的所有图片
const inferenceAll = async () => {
  if (!inferenceConfig.value) {
    message.warning("请先配置推理参数");
    showInferenceSettings.value = true;
    return;
  }
  if (imageFiles.value.length === 0) {
    message.warning("没有可推理的图片");
    return;
  }

  const startIndex = currentImageIndex.value >= 0 ? currentImageIndex.value : 0;
  const count = imageFiles.value.length - startIndex;
  await runInference(startIndex, count);
};

// 执行推理
const runInference = async (startIndex, count) => {
  // 创建 loading 消息
  const loadingMsg = message.loading(`正在推理 ${count} 张图片...`, {
    duration: 0,
  });

  try {
    const config = {
      mode:
        inferenceConfig.value.mode === "api"
          ? {
              type: "api",
              base_url: inferenceConfig.value.api.baseUrl,
              conf_threshold: inferenceConfig.value.api.confThreshold,
              iou_threshold: inferenceConfig.value.api.iouThreshold,
            }
          : {
              type: "onnx",
              model_path: inferenceConfig.value.onnx.modelPath,
              conf_threshold: inferenceConfig.value.onnx.confThreshold,
              iou_threshold: inferenceConfig.value.onnx.iouThreshold,
              use_gpu: inferenceConfig.value.onnx.useGpu,
            },
      count: {
        type: "count",
        value: count,
      },
    };

    const result = await invoke("inference_batch", {
      imagePaths: imageFiles.value,
      startIndex,
      count,
      config,
    });

    // 销毁 loading 消息
    loadingMsg.destroy();

    // 显示成功消息
    message.success(`推理完成！成功: ${result.success_count} 张，失败: ${result.error_count} 张`);

    // 刷新当前图片
    if (currentImageIndex.value >= 0) {
      await loadImage(currentImageIndex.value);
    }
  } catch (error) {
    console.error("推理失败:", error);

    // 销毁 loading 消息
    loadingMsg.destroy();

    // 显示错误消息
    message.error(`推理失败: ${error}`);
  }
};

// 初始化键盘管理器
const initKeyboardManager = () => {
  const callbacks = {
    openFolder,
    save: saveAnnotations,
    selectTool,
    rectangleTool,
    rotatedRectangleTool,
    createAnnotation, // 快捷键 N
    prevImage,
    nextImage,
    firstImage,
    lastImage,
    deleteAnnotation: deleteSelectedAnnotation,
    clearSelection,
    showHelp: toggleKeyboardHelp,
    toggleAnnotations, // 快捷键 H
  };

  keyboardManager.value = createKeyboardManager(callbacks);
  keyboardManager.value.startListening();
};

onMounted(() => {
  // 初始化应用
  initKeyboardManager();
});

onUnmounted(() => {
  if (keyboardManager.value) {
    keyboardManager.value.destroy();
  }

  // 清除自动保存定时器
  if (autoSaveTimer.value) {
    clearTimeout(autoSaveTimer.value);
  }

  // 如果有未保存的更改，尝试保存
  if (hasUnsavedChanges.value) {
    saveAnnotations();
  }
});
</script>

<template>
  <n-config-provider :theme="null">
    <div class="annotation-app">
      <!-- 顶部工具栏 -->
      <ToolBar
        :current-tool="currentTool"
        :save-status="saveStatus"
        :last-save-time="lastSaveTime"
        :show-annotations="showAnnotations"
        :has-image="!!currentImage"
        :inference-configured="inferenceConfig !== null"
        @switch-tool="switchTool"
        @open-folder="openFolder"
        @save="saveAnnotations"
        @show-help="toggleKeyboardHelp"
        @toggle-annotations="toggleAnnotations"
        @export-coco="exportToCoco"
        @export-yolo="exportToYolo"
        @import-coco="importFromCoco"
        @show-inference-settings="showInferenceSettingsDialog"
        @inference-one="inferenceOne"
        @inference-all="inferenceAll"
      />

      <div class="main-content">
        <!-- 左侧文件列表 -->
        <div class="sidebar left">
          <FileList
            :files="imageFiles"
            :current-index="currentImageIndex"
            @select-file="loadImage"
          />
        </div>

        <!-- 中间画布区域 -->
        <div class="canvas-area">
          <AnnotationCanvas
            :image="currentImage"
            :annotations="annotations"
            :selected-annotation="selectedAnnotation"
            :current-tool="currentTool"
            :category-colors="categoryColors"
            :label-categories="labelCategories"
            :show-annotations="showAnnotations"
            @add-annotation="addAnnotation"
            @update-annotation="updateAnnotation"
            @select-annotation="selectAnnotation"
            @switch-tool="switchTool"
          />
        </div>

        <!-- 右侧属性面板 -->
        <div class="sidebar right">
          <PropertyPanel
            :selected-annotation="selectedAnnotation"
            :annotations="annotations"
            :label-categories="labelCategories"
            :category-colors="categoryColors"
            @update-annotation="updateAnnotation"
            @delete-annotation="deleteAnnotation"
            @select-annotation="selectAnnotation"
          />
        </div>
      </div>

      <!-- 键盘快捷键帮助 -->
      <KeyboardHelp :visible="showKeyboardHelp" @close="showKeyboardHelp = false" />

      <!-- 推理设置对话框 -->
      <InferenceSettings
        :visible="showInferenceSettings"
        @close="showInferenceSettings = false"
        @settings-saved="handleSettingsSaved"
      />

      <!-- 错误提示 -->
      <div v-if="errorMessage" class="error-toast">
        {{ errorMessage }}
      </div>
    </div>
  </n-config-provider>
</template>

<style scoped>
.annotation-app {
  height: 100vh;
  display: flex;
  flex-direction: column;
  background-color: #f5f5f5 !important;
}

.main-content {
  flex: 1;
  display: flex;
  overflow: hidden;
  background-color: #f5f5f5 !important;
}

.sidebar {
  width: 280px;
  background-color: #ffffff !important;
  border: 1px solid #e0e0e0 !important;
  display: flex;
  flex-direction: column;
}

.sidebar.left {
  border-right: none;
}

.sidebar.right {
  border-left: none;
}

.canvas-area {
  flex: 1;
  background-color: #f5f5f5 !important;
  position: relative;
  overflow: hidden;
}

.error-toast {
  position: fixed;
  top: 20px;
  right: 20px;
  background-color: #f44336 !important;
  color: #ffffff !important;
  padding: 12px 16px;
  border-radius: 6px;
  font-size: 14px;
  z-index: 1000;
  max-width: 300px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  animation: slideIn 0.3s ease-out;
}

@keyframes slideIn {
  from {
    transform: translateX(100%);
    opacity: 0;
  }
  to {
    transform: translateX(0);
    opacity: 1;
  }
}
</style>
