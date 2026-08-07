<script setup lang="ts">
import { ref, computed, watch, onMounted, onBeforeUnmount } from "vue";
import { NButton, NInput, NSpace, NIcon, useMessage } from "naive-ui";
import { useAiStore } from "../../stores/ai";
import type { Message } from "../../types/ai";

const props = defineProps<{
  visible: boolean;
  contextText: string;
  cursorPosition: { from: number; to: number } | null;
  mode?: "complete" | "continue" | "rewrite" | "expand" | "polish";
}>();

const emit = defineEmits<{
  accept: [content: string];
  reject: [];
  close: [];
}>();

const aiStore = useAiStore();
const message = useMessage();

const userInput = ref("");
const isLoading = ref(false);

// 根据模式显示不同的标题
const modeTitle = computed(() => {
  const titles = {
    complete: "AI 补全助手",
    continue: "AI 续写",
    rewrite: "AI 改写",
    expand: "AI 扩写",
    polish: "AI 润色",
  };
  return titles[props.mode || "complete"];
});

// 构建 AI 请求
async function requestCompletion() {
  if (!userInput.value.trim()) {
    message.warning("请输入 AI 指令");
    return;
  }

  isLoading.value = true;
  aiStore.streamContent = "";
  aiStore.streamError = null;

  try {
    // 获取默认 provider
    const provider = aiStore.defaultProvider;
    if (!provider) {
      message.error("未配置 AI Provider，请先在设置中添加");
      isLoading.value = false;
      return;
    }

    // 根据模式构建不同的提示词
    const modePrompts = {
      complete: `指令：${userInput.value}\n\n上下文：\n${props.contextText}\n\n请根据指令和上下文生成内容：`,
      continue: `请继续续写以下内容，保持风格和情节的连贯性：\n\n${props.contextText}\n\n用户补充指令：${userInput.value}`,
      rewrite: `请改写以下内容，使其更加生动、流畅：\n\n${props.contextText}\n\n用户补充指令：${userInput.value}`,
      expand: `请扩写以下内容，增加细节描写和情节发展：\n\n${props.contextText}\n\n用户补充指令：${userInput.value}`,
      polish: `请润色以下内容，优化语言表达和文学性：\n\n${props.contextText}\n\n用户补充指令：${userInput.value}`,
    };

    // 构建消息
    const messages: Message[] = [
      {
        role: "system",
        content: "你是一位专业的小说创作助手。请根据用户的指令和上下文，生成高质量的文本内容。直接输出文本，不要包含任何解释或标记。",
      },
      {
        role: "user",
        content: modePrompts[props.mode || "complete"],
      },
    ];

    // 调用流式 API
    await aiStore.completeStream({
      provider_id: provider.id,
      model: provider.models_json || "gpt-3.5-turbo",
      messages,
      temperature: 0.7,
      max_tokens: 1000,
      stream: true,
    });

  } catch (error: any) {
    message.error(error?.message || "AI 补全失败");
  } finally {
    isLoading.value = false;
  }
}

// 接受补全
function acceptCompletion() {
  if (aiStore.streamContent) {
    emit("accept", aiStore.streamContent);
  }
}

// 拒绝补全
function rejectCompletion() {
  emit("reject");
}

// 关闭面板
function closePanel() {
  emit("close");
}

// 监听 ESC 键
function handleKeydown(e: KeyboardEvent) {
  if (e.key === "Escape") {
    closePanel();
  } else if (e.key === "Enter" && e.ctrlKey) {
    requestCompletion();
  }
}

onMounted(() => {
  window.addEventListener("keydown", handleKeydown);
});

onBeforeUnmount(() => {
  window.removeEventListener("keydown", handleKeydown);
});

// 监听 visible 变化，重置状态
watch(() => props.visible, (newVal) => {
  if (newVal) {
    userInput.value = "";
    aiStore.streamContent = "";
    aiStore.streamError = null;
  }
});
</script>

<template>
  <Teleport to="body">
    <Transition name="fade">
      <div v-if="visible" class="ai-completion-overlay" @click.self="closePanel">
        <div class="ai-completion-panel">
          <!-- 头部 -->
          <div class="panel-header">
            <div class="title">
              <NIcon size="20" color="#6366f1">
                <span class="i-carbon-ai" />
              </NIcon>
              <span>{{ modeTitle }}</span>
            </div>
            <NButton quaternary circle size="small" @click="closePanel">
              <template #icon>
                <span class="i-carbon-close" />
              </template>
            </NButton>
          </div>

          <!-- 输入区 -->
          <div class="input-section">
            <NInput
              v-model:value="userInput"
              type="textarea"
              placeholder="输入 AI 指令，例如：继续写这段对话...&#10;&#10;快捷键：Ctrl+Enter 发送，Esc 关闭"
              :rows="3"
              :disabled="isLoading"
              @keydown.ctrl.enter="requestCompletion"
            />
            <div class="input-actions">
              <NButton
                type="primary"
                :loading="isLoading"
                :disabled="!userInput.trim()"
                @click="requestCompletion"
              >
                {{ isLoading ? "生成中..." : "生成" }}
              </NButton>
            </div>
          </div>

          <!-- 结果区 -->
          <div v-if="aiStore.streamContent || aiStore.streamError" class="result-section">
            <div class="result-label">生成结果：</div>
            <div v-if="aiStore.streamError" class="error-message">
              {{ aiStore.streamError }}
            </div>
            <div v-else class="result-content">
              <div class="content-text">{{ aiStore.streamContent }}</div>
              <div v-if="isLoading" class="loading-indicator">
                <span class="dot" />
                <span class="dot" />
                <span class="dot" />
              </div>
            </div>
          </div>

          <!-- 操作区 -->
          <div v-if="aiStore.streamContent && !isLoading" class="action-section">
            <NSpace justify="end">
              <NButton @click="rejectCompletion">
                拒绝
              </NButton>
              <NButton type="primary" @click="acceptCompletion">
                接受
              </NButton>
            </NSpace>
          </div>

          <!-- 提示 -->
          <div class="tips">
            <span class="i-carbon-information" />
            <span>提示：提供清晰的指令和上下文，AI 会生成更符合预期的内容</span>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.ai-completion-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9999;
  animation: fadeIn 0.2s ease-out;
}

.ai-completion-panel {
  background: white;
  border-radius: 12px;
  width: 600px;
  max-width: 90vw;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
  animation: slideUp 0.3s ease-out;
}

:root.dark .ai-completion-panel {
  background: #1f1f1f;
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid #e5e7eb;
}

:root.dark .panel-header {
  border-bottom-color: #374151;
}

.title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 16px;
  font-weight: 600;
}

.input-section {
  padding: 20px;
  border-bottom: 1px solid #e5e7eb;
}

:root.dark .input-section {
  border-bottom-color: #374151;
}

.input-actions {
  display: flex;
  justify-content: flex-end;
  margin-top: 12px;
}

.result-section {
  padding: 20px;
  flex: 1;
  overflow-y: auto;
  min-height: 200px;
  max-height: 400px;
}

.result-label {
  font-size: 14px;
  font-weight: 500;
  margin-bottom: 12px;
  color: #6b7280;
}

:root.dark .result-label {
  color: #9ca3af;
}

.error-message {
  padding: 12px;
  background: #fef2f2;
  border: 1px solid #fecaca;
  border-radius: 6px;
  color: #dc2626;
  font-size: 14px;
}

:root.dark .error-message {
  background: #7f1d1d;
  border-color: #991b1b;
  color: #fca5a5;
}

.result-content {
  background: #f9fafb;
  border: 1px solid #e5e7eb;
  border-radius: 6px;
  padding: 16px;
  font-size: 14px;
  line-height: 1.6;
  white-space: pre-wrap;
  word-break: break-word;
}

:root.dark .result-content {
  background: #111827;
  border-color: #374151;
}

.content-text {
  color: #1f2937;
}

:root.dark .content-text {
  color: #f3f4f6;
}

.loading-indicator {
  display: inline-flex;
  gap: 4px;
  margin-top: 8px;
}

.dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: #6366f1;
  animation: pulse 1.4s infinite;
}

.dot:nth-child(2) {
  animation-delay: 0.2s;
}

.dot:nth-child(3) {
  animation-delay: 0.4s;
}

.action-section {
  padding: 16px 20px;
  border-top: 1px solid #e5e7eb;
  background: #f9fafb;
}

:root.dark .action-section {
  border-top-color: #374151;
  background: #111827;
}

.tips {
  padding: 12px 20px;
  background: #eff6ff;
  border-top: 1px solid #e5e7eb;
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  color: #6b7280;
}

:root.dark .tips {
  background: #1e3a8a;
  border-top-color: #374151;
  color: #9ca3af;
}

/* 动画 */
@keyframes fadeIn {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

@keyframes slideUp {
  from {
    transform: translateY(20px);
    opacity: 0;
  }
  to {
    transform: translateY(0);
    opacity: 1;
  }
}

@keyframes pulse {
  0%, 80%, 100% {
    opacity: 0.3;
    transform: scale(0.8);
  }
  40% {
    opacity: 1;
    transform: scale(1);
  }
}

/* 过渡动画 */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
