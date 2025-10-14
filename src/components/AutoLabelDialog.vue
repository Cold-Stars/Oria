<script setup>
import { ref, computed, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { createDiscreteApi } from "naive-ui";

const { message } = createDiscreteApi(["message"]);

const emit = defineEmits(["close", "inference-complete"]);

const props = defineProps({
  visible: {
    type: Boolean,
    default: false,
  },
  currentImageIndex: {
    type: Number,
    default: 0,
  },
  totalImages: {
    type: Number,
    default: 0,
  },
  imageFiles: {
    type: Array,
    default: () => [],
  },
});

// 推理模式: api 或 onnx
const inferenceMode = ref("api");

// API配置
const apiConfig = ref({
  baseUrl: "http://localhost:8000",
  confThreshold: 0.25,
  iouThreshold: 0.45,
});

// ONNX配置
const onnxConfig = ref({
  modelPath: "",
  confThreshold: 0.25,
  iouThreshold: 0.45,
  useGpu: false,
});

// ONNX模型状态
const onnxModelStatus = ref(null); // null, 'loading', 'loaded', 'error'
const onnxModelMessage = ref("");
const onnxModelInfo = ref(null);

// 推理数量配置 - 默认推理当前及之后所有图片
const countMode = ref("all"); // 固定为 'all'

// 状态
const isInferencing = ref(false);
const apiHealthStatus = ref(null); // null, 'checking', 'healthy', 'error'
const apiHealthMessage = ref("");
const modelInfo = ref(null);

// 进度
const progress = ref(0);
const currentInferenceIndex = ref(0);

// 计算实际要推理的数量 - 始终推理当前及之后的所有图片
const actualCount = computed(() => {
  const remaining = props.totalImages - props.currentImageIndex;
  return remaining;
});

// 推理按钮文本
const inferenceButtonText = computed(() => {
  if (isInferencing.value) {
    return `推理中... (${currentInferenceIndex.value}/${actualCount.value})`;
  }
  return `开始推理 (${actualCount.value}张)`;
});

// 检查API健康状态
const checkApiHealth = async () => {
  apiHealthStatus.value = "checking";
  apiHealthMessage.value = "正在连接...";

  try {
    const isHealthy = await invoke("check_api_health", {
      baseUrl: apiConfig.value.baseUrl,
    });

    if (isHealthy) {
      apiHealthStatus.value = "healthy";
      apiHealthMessage.value = "服务正常";

      // 获取模型信息
      try {
        const info = await invoke("get_api_model_info", {
          baseUrl: apiConfig.value.baseUrl,
        });
        modelInfo.value = info;
      } catch (error) {
        console.error("获取模型信息失败:", error);
      }
    } else {
      apiHealthStatus.value = "error";
      apiHealthMessage.value = "服务异常";
    }
  } catch (error) {
    console.error("健康检查失败:", error);
    apiHealthStatus.value = "error";
    apiHealthMessage.value = `连接失败: ${error}`;
  }
};

// 防抖定时器
let healthCheckTimeout = null;

// 当API URL变化时，自动检查健康状态（防抖）
watch(
  () => apiConfig.value.baseUrl,
  () => {
    if (props.visible && inferenceMode.value === "api" && !isInferencing.value) {
      // 清除之前的定时器
      if (healthCheckTimeout) {
        clearTimeout(healthCheckTimeout);
      }
      // 延迟500ms执行，避免频繁调用
      healthCheckTimeout = setTimeout(() => {
        checkApiHealth();
      }, 500);
    }
  }
);

// 当对话框打开时，检查API健康状态
watch(
  () => props.visible,
  (newVal, oldVal) => {
    console.log("对话框可见性变化:", oldVal, "=>", newVal);
    if (newVal && !oldVal && inferenceMode.value === "api" && !isInferencing.value) {
      // 只在从不可见变为可见时检查
      checkApiHealth();
    }
  }
);

// 选择ONNX模型文件
const selectOnnxModel = async () => {
  try {
    const path = await invoke("select_onnx_model");
    onnxConfig.value.modelPath = path;

    // 验证模型
    await validateOnnxModel();
  } catch (error) {
    if (
      !error.toString().includes("未选择文件") &&
      !error.toString().includes("No file selected")
    ) {
      console.error("选择模型失败:", error);
      message.error(`选择模型失败: ${error}`);
    }
  }
};

// 验证ONNX模型
const validateOnnxModel = async () => {
  if (!onnxConfig.value.modelPath) {
    return;
  }

  onnxModelStatus.value = "loading";
  onnxModelMessage.value = "正在加载模型...";

  try {
    const info = await invoke("validate_onnx_model", {
      modelPath: onnxConfig.value.modelPath,
    });

    onnxModelStatus.value = "loaded";
    onnxModelMessage.value = "模型加载成功";
    onnxModelInfo.value = info;
  } catch (error) {
    onnxModelStatus.value = "error";
    onnxModelMessage.value = `模型加载失败: ${error}`;
    onnxModelInfo.value = null;
  }
};

// 开始推理
const startInference = async () => {
  if (actualCount.value <= 0) {
    message.warning("没有图片可以推理");
    return;
  }

  // 检查推理模式的可用性
  if (inferenceMode.value === "api") {
    if (apiHealthStatus.value !== "healthy") {
      message.error("API服务不可用，请检查连接");
      return;
    }
  } else if (inferenceMode.value === "onnx") {
    if (onnxModelStatus.value !== "loaded") {
      message.error("ONNX模型未加载，请选择并验证模型");
      return;
    }
  }

  isInferencing.value = true;
  progress.value = 0;
  currentInferenceIndex.value = 0;

  try {
    // 构建推理配置
    const config = {
      mode:
        inferenceMode.value === "api"
          ? {
              type: "api",
              base_url: apiConfig.value.baseUrl,
              conf_threshold: apiConfig.value.confThreshold,
              iou_threshold: apiConfig.value.iouThreshold,
            }
          : {
              type: "onnx",
              model_path: onnxConfig.value.modelPath,
              conf_threshold: onnxConfig.value.confThreshold,
              iou_threshold: onnxConfig.value.iouThreshold,
              use_gpu: onnxConfig.value.useGpu,
            },
      count: {
        type: "count",
        value: actualCount.value,
      },
    };

    // 调用批量推理
    const result = await invoke("inference_batch", {
      imagePaths: props.imageFiles,
      startIndex: props.currentImageIndex,
      count: actualCount.value,
      config: config,
    });

    message.success(`推理完成！成功: ${result.success_count}, 失败: ${result.error_count}`);

    // 通知父组件刷新
    emit("inference-complete", result);

    // 推理成功后重置状态并关闭对话框
    isInferencing.value = false;
    progress.value = 0;
    currentInferenceIndex.value = 0;
    emit("close");
  } catch (error) {
    console.error("推理失败:", error);
    message.error(`推理失败: ${error}`);

    // 推理失败后重置状态
    isInferencing.value = false;
    progress.value = 0;
    currentInferenceIndex.value = 0;

    // 推理失败后重新检查API健康状态（仅API模式，且对话框仍打开）
    if (inferenceMode.value === "api" && props.visible) {
      checkApiHealth();
    }
  }
};

// 关闭对话框
const closeDialog = () => {
  if (!isInferencing.value) {
    emit("close");
  }
};
</script>

<template>
  <n-modal :show="visible" :mask-closable="!isInferencing" @update:show="closeDialog">
    <n-card
      style="width: 600px"
      title="模型自动标注"
      :bordered="false"
      size="huge"
      role="dialog"
      aria-modal="true"
    >
      <template #header-extra>
        <n-button text @click="closeDialog" :disabled="isInferencing" style="font-size: 20px">
          ✕
        </n-button>
      </template>

      <n-space vertical :size="20">
        <!-- 推理模式选择 -->
        <n-form-item label="推理模式">
          <n-radio-group v-model:value="inferenceMode" :disabled="isInferencing">
            <n-radio value="api"> API推理 </n-radio>
            <n-radio value="onnx"> ONNX推理 </n-radio>
          </n-radio-group>
        </n-form-item>

        <!-- API模式配置 -->
        <div v-if="inferenceMode === 'api'">
          <n-space vertical :size="12">
            <n-form-item label="API地址">
              <n-input
                v-model:value="apiConfig.baseUrl"
                placeholder="http://localhost:8000"
                :disabled="isInferencing"
              />
            </n-form-item>

            <n-space align="center">
              <n-button
                size="small"
                @click="checkApiHealth"
                :loading="apiHealthStatus === 'checking'"
                :disabled="isInferencing"
              >
                测试连接
              </n-button>
              <n-tag v-if="apiHealthStatus === 'healthy'" type="success" size="small">
                {{ apiHealthMessage }}
              </n-tag>
              <n-tag v-else-if="apiHealthStatus === 'error'" type="error" size="small">
                {{ apiHealthMessage }}
              </n-tag>
              <n-tag v-else-if="apiHealthStatus === 'checking'" size="small">
                {{ apiHealthMessage }}
              </n-tag>
            </n-space>

            <!-- 模型信息 -->
            <div v-if="modelInfo" class="model-info">
              <n-text depth="3" style="font-size: 12px">
                模型: {{ modelInfo.model_name }} | 类别数: {{ modelInfo.class_names.length }} |
                输入尺寸:
                {{ modelInfo.input_size.join("x") }}
              </n-text>
            </div>

            <n-form-item label="置信度阈值">
              <n-slider
                v-model:value="apiConfig.confThreshold"
                :step="0.05"
                :min="0.1"
                :max="0.9"
                :disabled="isInferencing"
              />
              <n-input-number
                v-model:value="apiConfig.confThreshold"
                :step="0.05"
                :min="0.1"
                :max="0.9"
                style="margin-left: 12px; width: 130px"
                :disabled="isInferencing"
              />
            </n-form-item>

            <n-form-item label="IOU阈值">
              <n-slider
                v-model:value="apiConfig.iouThreshold"
                :step="0.05"
                :min="0.1"
                :max="0.9"
                :disabled="isInferencing"
              />
              <n-input-number
                v-model:value="apiConfig.iouThreshold"
                :step="0.05"
                :min="0.1"
                :max="0.9"
                style="margin-left: 12px; width: 130px"
                :disabled="isInferencing"
              />
            </n-form-item>
          </n-space>
        </div>

        <!-- ONNX模式配置 -->
        <div v-if="inferenceMode === 'onnx'">
          <n-space vertical :size="12">
            <n-form-item label="模型文件">
              <n-space style="width: 100%">
                <n-input
                  v-model:value="onnxConfig.modelPath"
                  placeholder="选择ONNX模型文件 (.onnx)"
                  :disabled="isInferencing"
                  readonly
                  style="flex: 1"
                />
                <n-button @click="selectOnnxModel" :disabled="isInferencing"> 选择模型 </n-button>
              </n-space>
            </n-form-item>

            <n-space align="center" v-if="onnxConfig.modelPath">
              <n-button
                size="small"
                @click="validateOnnxModel"
                :loading="onnxModelStatus === 'loading'"
                :disabled="isInferencing"
              >
                验证模型
              </n-button>
              <n-tag v-if="onnxModelStatus === 'loaded'" type="success" size="small">
                {{ onnxModelMessage }}
              </n-tag>
              <n-tag v-else-if="onnxModelStatus === 'error'" type="error" size="small">
                {{ onnxModelMessage }}
              </n-tag>
              <n-tag v-else-if="onnxModelStatus === 'loading'" size="small">
                {{ onnxModelMessage }}
              </n-tag>
            </n-space>

            <!-- 模型信息 -->
            <div v-if="onnxModelInfo" class="model-info">
              <n-text depth="3" style="font-size: 12px">
                类别数: {{ onnxModelInfo.class_names.length }} | 输入尺寸:
                {{ onnxModelInfo.input_size.join("x") }}
              </n-text>
            </div>

            <n-alert v-if="!onnxConfig.modelPath" type="info" title="提示">
              请选择一个ONNX格式的YOLOv8模型文件
            </n-alert>

            <n-form-item label="置信度阈值">
              <n-slider
                v-model:value="onnxConfig.confThreshold"
                :step="0.05"
                :min="0.1"
                :max="0.9"
                :disabled="isInferencing"
              />
              <n-input-number
                v-model:value="onnxConfig.confThreshold"
                :step="0.05"
                :min="0.1"
                :max="0.9"
                style="margin-left: 12px; width: 120px"
                :disabled="isInferencing"
              />
            </n-form-item>

            <n-form-item label="IOU阈值">
              <n-slider
                v-model:value="onnxConfig.iouThreshold"
                :step="0.05"
                :min="0.1"
                :max="0.9"
                :disabled="isInferencing"
              />
              <n-input-number
                v-model:value="onnxConfig.iouThreshold"
                :step="0.05"
                :min="0.1"
                :max="0.9"
                style="margin-left: 12px; width: 120px"
                :disabled="isInferencing"
              />
            </n-form-item>

            <n-form-item label="使用GPU加速">
              <n-space align="center">
                <n-switch v-model:value="onnxConfig.useGpu" :disabled="isInferencing" />
                <n-text depth="3" style="font-size: 12px">
                  {{ onnxConfig.useGpu ? "GPU (CUDA)" : "CPU" }}
                </n-text>
              </n-space>
              <n-text depth="3" style="font-size: 11px; margin-top: 4px">
                注意: GPU推理需要安装CUDA和对应的ONNX Runtime GPU版本
              </n-text>
            </n-form-item>
          </n-space>
        </div>

        <n-divider />

        <!-- 推理数量信息 -->
        <n-alert type="info" :bordered="false">
          <template #icon>
            <span style="font-size: 16px">📊</span>
          </template>
          <div style="line-height: 1.6">
            <div>
              将从第 <n-text strong>{{ currentImageIndex + 1 }}</n-text> 张图片开始推理
            </div>
            <div style="margin-top: 4px">
              共推理 <n-text strong type="primary">{{ actualCount }}</n-text> 张图片
              <n-text depth="3" style="font-size: 12px"> （当前及之后的所有图片） </n-text>
            </div>
          </div>
        </n-alert>

        <!-- 进度条 -->
        <div v-if="isInferencing">
          <n-progress type="line" :percentage="progress" :indicator-placement="'inside'" />
          <n-text depth="3" style="font-size: 12px; margin-top: 8px">
            正在推理: {{ currentInferenceIndex }} / {{ actualCount }}
          </n-text>
        </div>
      </n-space>

      <template #footer>
        <n-space justify="end">
          <n-button @click="closeDialog" :disabled="isInferencing"> 取消 </n-button>
          <n-button
            type="primary"
            @click="startInference"
            :disabled="
              isInferencing ||
              (inferenceMode === 'api' && apiHealthStatus !== 'healthy') ||
              (inferenceMode === 'onnx' && onnxModelStatus !== 'loaded') ||
              actualCount <= 0
            "
            :loading="isInferencing"
          >
            {{ inferenceButtonText }}
          </n-button>
        </n-space>
      </template>
    </n-card>
  </n-modal>
</template>

<style scoped>
.model-info {
  padding: 8px 12px;
  background-color: #f5f5f5;
  border-radius: 4px;
  margin-top: 8px;
}
</style>
