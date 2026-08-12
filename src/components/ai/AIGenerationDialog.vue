<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { NModal, NSelect, NInput, NInputNumber, NButton, NSpace, NAlert, NSpin, NTag, NPopconfirm } from 'naive-ui';
import { useAiGenerationStore } from '../../stores/ai_generation';
import { useChapterStore } from '../../stores/chapter';
import type { GenerateRequest } from '../../types/ai_generation';

const props = defineProps<{
  show: boolean;
}>();

const emit = defineEmits<{
  'update:show': [value: boolean];
  'apply': [content: string];
}>();

const aiGenerationStore = useAiGenerationStore();
const chapterStore = useChapterStore();

// 表单数据
const form = ref<GenerateRequest>({
  chapter_id: '',
  mode: 'continuation',
  user_instruction: '',
  parameters: {
    target_words: 2000,
    temperature: 0.7,
  },
});

// 模式选项
const modeOptions = [
  { label: '续写', value: 'continuation' },
  { label: '扩写', value: 'expansion' },
  { label: '改写', value: 'rewrite' },
  { label: '润色', value: 'polish' },
  { label: '大纲生成', value: 'outline' },
];

// 计算属性
const activeChapterId = computed(() => chapterStore.activeChapterId);
const canGenerate = computed(() => {
  return activeChapterId.value && !aiGenerationStore.loading;
});

// 监听对话框显示
watch(() => props.show, async (newVal) => {
  if (newVal && activeChapterId.value) {
    form.value.chapter_id = activeChapterId.value;
    aiGenerationStore.loadHistory(activeChapterId.value);
  }
});

// 生成
async function handleGenerate() {
  if (!canGenerate.value) return;
  
  try {
    await aiGenerationStore.generate(form.value);
  } catch (e) {
    console.error('生成失败:', e);
  }
}

// 应用生成结果
function handleApply() {
  if (aiGenerationStore.currentGeneration) {
    emit('apply', aiGenerationStore.currentGeneration.content);
    handleClose();
  }
}

// 关闭对话框
function handleClose() {
  emit('update:show', false);
  aiGenerationStore.clearCurrentGeneration();
}

// 删除历史记录
async function handleDeleteHistory(id: string) {
  try {
    await aiGenerationStore.deleteHistory(id);
  } catch (e) {
    console.error('删除失败:', e);
  }
}

// 清空历史
async function handleClearHistory() {
  if (!activeChapterId.value) return;
  try {
    await aiGenerationStore.clearHistory(activeChapterId.value);
  } catch (e) {
    console.error('清空失败:', e);
  }
}

// 格式化时间
function formatTime(time: string): string {
  const date = new Date(time);
  return date.toLocaleString('zh-CN');
}
</script>

<template>
  <NModal
    :show="show"
    preset="card"
    title="AI 生成"
    style="width: 800px; max-width: 90vw;"
    :mask-closable="true"
    @update:show="handleClose"
  >
    <div class="ai-generation-dialog">
      <!-- 左侧：生成表单 -->
      <div class="generation-form">
        <div class="form-section">
          <div class="form-label">生成模式</div>
          <NSelect
            v-model:value="form.mode"
            :options="modeOptions"
            placeholder="选择生成模式"
          />
        </div>

        <div class="form-section">
          <div class="form-label">用户指令（可选）</div>
          <NInput
            v-model:value="form.user_instruction"
            type="textarea"
            :rows="3"
            placeholder="输入具体的生成要求，例如：保持轻松愉快的语气..."
          />
        </div>

        <div class="form-section">
          <div class="form-label">生成参数</div>
          <div class="parameter-row">
            <span class="parameter-label">期望章节字数：</span>
            <NInputNumber
              v-model:value="form.parameters!.target_words"
              :min="100"
              :max="50000"
              :step="100"
              style="width: 120px;"
            />
          </div>
          <div class="parameter-row">
            <span class="parameter-label">创造性：</span>
            <NInputNumber
              v-model:value="form.parameters!.temperature"
              :min="0"
              :max="1"
              :step="0.1"
              style="width: 120px;"
            />
          </div>
        </div>

        <NSpace vertical>
          <NButton
            type="primary"
            block
            :loading="aiGenerationStore.loading"
            :disabled="!canGenerate"
            @click="handleGenerate"
          >
            开始生成
          </NButton>

          <NAlert
            v-if="aiGenerationStore.error"
            type="error"
            :title="aiGenerationStore.error"
          />
        </NSpace>
      </div>

      <!-- 右侧：生成结果和历史 -->
      <div class="generation-result">
        <!-- 当前生成结果 -->
        <div v-if="aiGenerationStore.currentGeneration" class="current-result">
          <div class="result-header">
            <span class="result-title">生成结果</span>
            <NButton
              type="primary"
              size="small"
              @click="handleApply"
            >
              应用到编辑器
            </NButton>
          </div>
          <div class="result-content">
            {{ aiGenerationStore.currentGeneration.content }}
          </div>
        </div>

        <!-- 历史记录 -->
        <div class="history-section">
          <div class="history-header">
            <span class="history-title">历史记录</span>
            <NPopconfirm
              v-if="aiGenerationStore.history.length > 0"
              @positive-click="handleClearHistory"
            >
              <template #trigger>
                <NButton size="small" quaternary type="error">
                  清空
                </NButton>
              </template>
              确定要清空所有历史记录吗？
            </NPopconfirm>
          </div>

          <NSpin :show="aiGenerationStore.loading && aiGenerationStore.history.length === 0">
            <div v-if="aiGenerationStore.history.length === 0" class="empty-history">
              暂无历史记录
            </div>
            <div v-else class="history-list">
              <div
                v-for="item in aiGenerationStore.history"
                :key="item.id"
                class="history-item"
              >
                <div class="history-item-header">
                  <NTag :bordered="false" size="small" type="info">
                    {{ modeOptions.find(m => m.value === item.mode)?.label || item.mode }}
                  </NTag>
                  <span class="history-time">{{ formatTime(item.created_at) }}</span>
                  <NButton
                    size="tiny"
                    quaternary
                    type="error"
                    @click="handleDeleteHistory(item.id)"
                  >
                    删除
                  </NButton>
                </div>
                <div class="history-content">
                  {{ item.output_content.substring(0, 100) }}{{ item.output_content.length > 100 ? '...' : '' }}
                </div>
              </div>
            </div>
          </NSpin>
        </div>
      </div>
    </div>
  </NModal>
</template>

<style scoped>
.ai-generation-dialog {
  display: flex;
  gap: 20px;
  height: 600px;
}

.generation-form {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 16px;
  overflow-y: auto;
  padding-right: 10px;
}

.form-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.form-label {
  font-weight: 500;
  font-size: 14px;
}

.parameter-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 8px;
}

.parameter-label {
  font-size: 13px;
  color: #666;
  min-width: 80px;
}

.generation-result {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 16px;
  overflow: hidden;
}

.current-result {
  display: flex;
  flex-direction: column;
  gap: 8px;
  border: 1px solid #e0e0e0;
  border-radius: 8px;
  padding: 12px;
  background: #f9f9f9;
}

.result-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.result-title {
  font-weight: 500;
  font-size: 14px;
}

.result-content {
  max-height: 200px;
  overflow-y: auto;
  font-size: 13px;
  line-height: 1.6;
  white-space: pre-wrap;
  word-break: break-word;
}

.history-section {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.history-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.history-title {
  font-weight: 500;
  font-size: 14px;
}

.empty-history {
  text-align: center;
  color: #999;
  padding: 40px 0;
  font-size: 13px;
}

.history-list {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.history-item {
  border: 1px solid #e0e0e0;
  border-radius: 6px;
  padding: 10px;
  background: #fafafa;
}

.history-item-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 6px;
}

.history-time {
  font-size: 12px;
  color: #999;
  flex: 1;
}

.history-content {
  font-size: 12px;
  color: #666;
  line-height: 1.5;
  white-space: pre-wrap;
  word-break: break-word;
}
</style>

<style>
/* 黑暗模式适配（非 scoped 以匹配全局 .dark 类） */
.dark .ai-generation-dialog .parameter-label {
  color: #9ca3af;
}

.dark .ai-generation-dialog .current-result {
  border-color: #374151;
  background: #1f2937;
}

.dark .ai-generation-dialog .empty-history {
  color: #6b7280;
}

.dark .ai-generation-dialog .history-item {
  border-color: #374151;
  background: #1f2937;
}

.dark .ai-generation-dialog .history-time {
  color: #6b7280;
}

.dark .ai-generation-dialog .history-content {
  color: #9ca3af;
}
</style>
