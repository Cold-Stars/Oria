<script setup>
import { ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { createDiscreteApi } from "naive-ui";

const { message } = createDiscreteApi(["message"]);

const emit = defineEmits(["close", "settings-saved"]);

const props = defineProps({
  visible: {
    type: Boolean,
    default: false,
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
const onnxModelStatus = ref(null);
const onnxModelMessage = ref("");
const onnxModelInfo = ref(null);

// 选择ONNX模型
const selectOnnxModel = async () => {
  try {
    const path = await invoke("select_onnx_model");
    if (path) {
      onnxConfig.value.modelPath = path;
      onnxModelStatus.value = null;
      onnxModelMessage.value = "";
      onnxModelInfo.value = null;
    }
  } catch (error) {
    console.error("选择模型失败:", error);
    message.error(`选择模型失败: ${error}`);
  }
};

// 验证ONNX模型
const validateOnnxModel = async () => {
  if (!onnxConfig.value.modelPath) {
    message.warning("请先选择模型文件");
    return;
  }

  onnxModelStatus.value = "loading";
  onnxModelMessage.value = "验证中...";

  try {
    const modelInfo = await invoke("validate_onnx_model", {
      modelPath: onnxConfig.value.modelPath,
    });

    onnxModelStatus.value = "loaded";
    onnxModelMessage.value = "模型验证成功";
    onnxModelInfo.value = modelInfo;
    message.success("模型验证成功");
  } catch (error) {
    console.error("模型验证失败:", error);
    onnxModelStatus.value = "error";
    onnxModelMessage.value = `验证失败: ${error}`;
    message.error(`模型验证失败: ${error}`);
  }
};

// 检查API健康状态
const checkApiHealth = async () => {
  try {
    message.loading("检查API连接...", { duration: 0, key: "api-check" });
    await invoke("check_api_health", {
      baseUrl: apiConfig.value.baseUrl,
    });
    message.success("API连接正常", { key: "api-check" });
  } catch (error) {
    console.error("API检查失败:", error);
    message.error(`API连接失败: ${error}`, { key: "api-check" });
  }
};

// 保存设置
const saveSettings = () => {
  const settings = {
    mode: inferenceMode.value,
    api: apiConfig.value,
    onnx: onnxConfig.value,
  };

  // 验证设置
  if (inferenceMode.value === "onnx" && !onnxConfig.value.modelPath) {
    message.warning("请选择 ONNX 模型文件");
    return;
  }

  emit("settings-saved", settings);
  message.success("设置已保存");
  emit("close");
};

// 监听模式切换
watch(inferenceMode, () => {
  onnxModelStatus.value = null;
  onnxModelMessage.value = "";
});
</script>

<template>
  <n-modal
    :show="visible"
    :mask-closable="true"
    @update:show="() => emit('close')"
  >
    <n-card
      style="width: 600px"
      title="推理参数设置"
      :bordered="false"
      size="huge"
      role="dialog"
      aria-modal="true"
    >
      <!-- <template #header-extra>
        <n-button quaternary circle @click="emit('close')">
          <template #icon>
            <n-icon><CloseOutline /></n-icon>
          </template>
        </n-button>
      </template> -->

      <n-space vertical :size="20">
        <!-- 推理模式选择 -->
        <n-form-item label="推理模式">
          <n-radio-group v-model:value="inferenceMode">
            <n-space>
              <n-radio value="api">API 推理</n-radio>
              <n-radio value="onnx">ONNX 本地推理</n-radio>
            </n-space>
          </n-radio-group>
        </n-form-item>

        <n-divider />

        <!-- API模式配置 -->
        <div v-if="inferenceMode === 'api'">
          <n-space vertical :size="12">
            <n-form-item label="API 地址">
              <n-input
                v-model:value="apiConfig.baseUrl"
                placeholder="http://localhost:8000"
              >
                <template #suffix>
                  <n-button text @click="checkApiHealth">测试连接</n-button>
                </template>
              </n-input>
            </n-form-item>

            <n-form-item label="置信度阈值">
              <div style="display: flex; align-items: center; gap: 12px; width: 100%">
                <n-slider
                  v-model:value="apiConfig.confThreshold"
                  :step="0.05"
                  :min="0.1"
                  :max="0.9"
                  style="flex: 1; min-width: 0"
                />
                <n-input-number
                  v-model:value="apiConfig.confThreshold"
                  :step="0.05"
                  :min="0.1"
                  :max="0.9"
                  :show-button="false"
                  style="width: 120px; flex-shrink: 0"
                />
              </div>
            </n-form-item>

            <n-form-item label="IOU阈值">
              <div style="display: flex; align-items: center; gap: 12px; width: 100%">
                <n-slider
                  v-model:value="apiConfig.iouThreshold"
                  :step="0.05"
                  :min="0.1"
                  :max="0.9"
                  style="flex: 1; min-width: 0"
                />
                <n-input-number
                  v-model:value="apiConfig.iouThreshold"
                  :step="0.05"
                  :min="0.1"
                  :max="0.9"
                  :show-button="false"
                  style="width: 120px; flex-shrink: 0"
                />
              </div>
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
                  readonly
                  style="flex: 1"
                />
                <n-button @click="selectOnnxModel">选择模型</n-button>
              </n-space>
            </n-form-item>

            <n-space align="center" v-if="onnxConfig.modelPath">
              <n-button
                size="small"
                @click="validateOnnxModel"
                :loading="onnxModelStatus === 'loading'"
              >
                验证模型
              </n-button>
              <n-tag
                v-if="onnxModelStatus === 'loaded'"
                type="success"
                size="small"
              >
                {{ onnxModelMessage }}
              </n-tag>
              <n-tag
                v-else-if="onnxModelStatus === 'error'"
                type="error"
                size="small"
              >
                {{ onnxModelMessage }}
              </n-tag>
            </n-space>

            <div v-if="onnxModelInfo" class="model-info">
              <n-text depth="3" style="font-size: 12px">
                类别数: {{ onnxModelInfo.class_names.length }} | 输入尺寸:
                {{ onnxModelInfo.input_size.join("x") }}
              </n-text>
            </div>

            <n-form-item label="置信度阈值">
              <div style="display: flex; align-items: center; gap: 12px; width: 100%">
                <n-slider
                  v-model:value="onnxConfig.confThreshold"
                  :step="0.05"
                  :min="0.1"
                  :max="0.9"
                  style="flex: 1; min-width: 0"
                />
                <n-input-number
                  v-model:value="onnxConfig.confThreshold"
                  :step="0.05"
                  :min="0.1"
                  :max="0.9"
                  :show-button="false"
                  style="width: 120px; flex-shrink: 0"
                />
              </div>
            </n-form-item>

            <n-form-item label="IOU阈值">
              <div style="display: flex; align-items: center; gap: 12px; width: 100%">
                <n-slider
                  v-model:value="onnxConfig.iouThreshold"
                  :step="0.05"
                  :min="0.1"
                  :max="0.9"
                  style="flex: 1; min-width: 0"
                />
                <n-input-number
                  v-model:value="onnxConfig.iouThreshold"
                  :step="0.05"
                  :min="0.1"
                  :max="0.9"
                  :show-button="false"
                  style="width: 120px; flex-shrink: 0"
                />
              </div>
            </n-form-item>

            <n-form-item label="使用GPU加速">
              <n-space align="center">
                <n-switch v-model:value="onnxConfig.useGpu" />
                <n-text depth="3" style="font-size: 12px">
                  {{ onnxConfig.useGpu ? "GPU (CUDA)" : "CPU" }}
                </n-text>
              </n-space>
              <n-text
                depth="3"
                style="font-size: 11px; margin-top: 4px; display: block"
              >
                注意: GPU推理需要安装CUDA和对应的ONNX Runtime GPU版本
              </n-text>
            </n-form-item>
          </n-space>
        </div>
      </n-space>

      <template #footer>
        <n-space justify="end">
          <n-button @click="emit('close')">取消</n-button>
          <n-button type="primary" @click="saveSettings">保存设置</n-button>
        </n-space>
      </template>
    </n-card>
  </n-modal>
</template>

<style scoped>
.model-info {
  padding: 8px 12px;
  background: rgba(0, 0, 0, 0.02);
  border-radius: 4px;
}
</style>
