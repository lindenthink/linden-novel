<script setup lang="ts">
import { ref, computed, watch, nextTick, h } from 'vue';
import { NModal, NSelect, NInput, NInputNumber, NButton, NSpace, NAlert, NSpin, NTag, NPopconfirm, NCollapse, NCollapseItem, useMessage, useDialog } from 'naive-ui';
import { useAiGenerationStore } from '../../stores/ai_generation';
import { useChapterStore } from '../../stores/chapter';
import { useEditorUI, type AIGenerationMode } from '../../composables/useEditorUI';
import type { GenerateRequest } from '../../types/ai_generation';
import type { Chapter } from '../../types';
import { listChapterElements } from '../../api/element';

const props = defineProps<{
  show: boolean;
}>();

const emit = defineEmits<{
  'update:show': [value: boolean];
  'apply': [content: string];
}>();

const aiGenerationStore = useAiGenerationStore();
const chapterStore = useChapterStore();
const { aiGenerationDefaultMode } = useEditorUI();
const message = useMessage();
const dialog = useDialog();

// 推理过程折叠状态：默认展开，便于实时观察；推理完成后自动收起
const reasoningExpanded = ref<string[]>(['reasoning']);

// 推理完成（reasoningActive 由 true 转为 false）时自动收起折叠区
watch(
  () => aiGenerationStore.reasoningActive,
  (active, prev) => {
    if (prev && !active) {
      reasoningExpanded.value = [];
    }
  },
);

// 内容区 ref：用于流式追加时自动滚动到底部
const reasoningContentRef = ref<HTMLElement | null>(null);
const resultContentRef = ref<HTMLElement | null>(null);

// 流式内容追加时自动滚动到底部（推理过程 / 正文）
watch(
  () => aiGenerationStore.streamReasoning,
  async () => {
    await nextTick();
    const el = reasoningContentRef.value;
    if (el) {
      el.scrollTop = el.scrollHeight;
    }
  },
);

watch(
  () => aiGenerationStore.streamContent,
  async () => {
    await nextTick();
    const el = resultContentRef.value;
    if (el) {
      el.scrollTop = el.scrollHeight;
    }
  },
);

// 表单数据
const form = ref<GenerateRequest>({
  chapter_id: '',
  mode: 'continuation',
  user_instruction: '',
  parameters: {
    target_words: 2000,
    temperature: 0.7,
    constraint: 'strict',
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

// 约束程度选项：宽松 token 消耗少、速度快但可能有 AI 痕迹；严格相反
const constraintOptions = [
  {
    label: '宽松',
    value: 'loose',
    description: 'token 消耗少，生成速度快，可能有 AI 生成痕迹',
  },
  {
    label: '严格',
    value: 'strict',
    description: 'token 消耗多，生成速度慢，尽量减少 AI 生成痕迹',
  },
];

// 菜单项渲染：主标签 + 灰色描述
function renderConstraintLabel(option: any) {
  return h('div', { style: 'display: flex; flex-direction: column; padding: 2px 0; white-space: normal; word-break: break-word;' }, [
    h('span', { style: 'font-size: 14px;' }, option.label),
    h('span', { style: 'font-size: 12px; color: #999; margin-top: 2px; line-height: 1.4;' }, option.description),
  ]);
}

// 选中后输入框只显示纯文本 label，不带描述
function renderConstraintTag(props: { option: any }) {
  return props.option.label;
}

// 计算属性
const activeChapterId = computed(() => chapterStore.activeChapterId);
const canGenerate = computed(() => {
  return activeChapterId.value && !aiGenerationStore.loading;
});

// 监听对话框显示：设置默认模式并加载历史
watch(
  () => props.show,
  async (newVal) => {
    if (newVal && activeChapterId.value) {
      form.value.chapter_id = activeChapterId.value;
      form.value.mode = aiGenerationDefaultMode.value as AIGenerationMode;
      form.value.user_instruction = '';
      aiGenerationStore.loadHistory(activeChapterId.value);

      // 首次打开时检查是否关联角色
      try {
        const elements = await listChapterElements(activeChapterId.value);
        const hasCharacter = elements.some(el => el.element_type === 'character');
        if (!hasCharacter) {
          message.warning('当前章节未关联角色，请先在侧边栏关联关键角色、故事线等元素后再使用 AI 生成。');
        }
      } catch (e) {
        console.error('加载章节元素失败:', e);
      }

      await nextTick();
    }
  }
);

// 找到当前章节的上一章（按 order_index 跨卷排序）
function findPreviousChapter(): Chapter | null {
  const currentId = chapterStore.activeChapterId;
  if (!currentId) return null;
  const sorted = [...chapterStore.chapters].sort((a, b) => a.order_index - b.order_index);
  const idx = sorted.findIndex((c) => c.id === currentId);
  if (idx <= 0) return null;
  return sorted[idx - 1];
}

// 检查上一章是否有内容但无摘要，若无则弹窗让用户确认是否继续
async function checkPrevChapterSummary(): Promise<boolean> {
  const prev = findPreviousChapter();
  if (!prev) return true;
  // 有正文内容但无摘要：续写提示词将缺少上一章上下文
  if (prev.word_count > 0 && !prev.summary) {
    return new Promise<boolean>((resolve) => {
      dialog.warning({
        title: '上一章缺少摘要',
        content: `上一章「${prev.title}」有正文内容但未生成摘要，AI 生成将缺少上一章的上下文，可能影响连贯性。是否继续？`,
        positiveText: '继续生成',
        negativeText: '取消',
        onPositiveClick: () => resolve(true),
        onNegativeClick: () => resolve(false),
        onClose: () => resolve(false),
        onMaskClick: () => resolve(false),
      });
    });
  }
  return true;
}

// 生成（流式：首 token 即实时追加显示）
async function handleGenerate() {
  if (!canGenerate.value) return;

  // 上一章有内容但无摘要时让用户确认
  const ok = await checkPrevChapterSummary();
  if (!ok) return;

  try {
    await aiGenerationStore.generateStream(form.value);
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

// 应用历史记录到编辑器
function handleApplyHistory(content: string) {
  emit('apply', content);
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
            <span class="parameter-label">约束程度：</span>
            <NSelect
              v-model:value="form.parameters!.constraint"
              :options="constraintOptions"
              :render-label="renderConstraintLabel"
              :render-tag="renderConstraintTag"
              :menu-props="{ style: 'min-width: 320px' }"
              style="width: 240px;"
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
        <!-- 推理过程（DeepSeek thinking 模式，仅在有推理内容时显示） -->
        <div
          v-if="aiGenerationStore.streamReasoning || aiGenerationStore.reasoningActive"
          class="reasoning-section"
        >
          <NCollapse v-model:expanded-names="reasoningExpanded" :default-expanded-names="['reasoning']">
            <NCollapseItem name="reasoning">
              <template #header>
                <span class="reasoning-title">
                  推理过程
                  <span v-if="aiGenerationStore.reasoningActive" class="reasoning-hint">
                    思考中<span class="cursor-blink">▌</span>
                  </span>
                  <span v-else class="reasoning-done-hint">已完成</span>
                </span>
              </template>
              <div ref="reasoningContentRef" class="reasoning-content">
                {{ aiGenerationStore.streamReasoning }}
                <span v-if="aiGenerationStore.reasoningActive" class="cursor-blink">▌</span>
              </div>
            </NCollapseItem>
          </NCollapse>
        </div>

        <!-- 当前生成结果 / 流式实时输出 -->
        <div
          v-if="aiGenerationStore.currentGeneration || aiGenerationStore.streaming || aiGenerationStore.streamContent"
          class="current-result"
        >
          <div class="result-header">
            <span class="result-title">
              生成结果
              <span v-if="aiGenerationStore.streaming && !aiGenerationStore.reasoningActive" class="streaming-hint">生成中…</span>
              <span v-else-if="aiGenerationStore.reasoningActive" class="streaming-hint">推理中…</span>
            </span>
            <NButton
              v-if="aiGenerationStore.currentGeneration && !aiGenerationStore.streaming"
              type="primary"
              size="small"
              @click="handleApply"
            >
              应用到编辑器
            </NButton>
          </div>
          <div ref="resultContentRef" class="result-content">
            <!-- 流式过程中显示 streamContent；流结束后显示 currentGeneration.content -->
            {{ aiGenerationStore.streaming ? aiGenerationStore.streamContent : aiGenerationStore.currentGeneration?.content }}
            <span v-if="aiGenerationStore.streaming && !aiGenerationStore.reasoningActive" class="cursor-blink">▌</span>
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
                    type="primary"
                    @click="handleApplyHistory(item.output_content)"
                  >
                    应用
                  </NButton>
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

.streaming-hint {
  margin-left: 8px;
  font-size: 12px;
  font-weight: normal;
  color: #18a058;
}

.cursor-blink {
  display: inline-block;
  color: #18a058;
  animation: blink 1s step-start infinite;
  margin-left: 1px;
}

@keyframes blink {
  50% { opacity: 0; }
}

/* 推理过程折叠区 */
.reasoning-section {
  padding: 12px 5px;
  margin-bottom: 12px;
  border: 1px solid var(--n-border-color, #e0e0e6);
  border-radius: 4px;
  background: var(--n-color, #fafafc);
}

.reasoning-title {
  font-size: 13px;
  font-weight: 500;
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.reasoning-hint {
  margin-left: 8px;
  font-size: 12px;
  color: #f0a020;
  display: inline-flex;
  align-items: center;
}

.reasoning-done-hint {
  margin-left: 8px;
  font-size: 12px;
  color: #909399;
}

.reasoning-content {
  max-height: 280px;
  overflow-y: auto;
  font-size: 12.5px;
  line-height: 1.6;
  color: #6b7280;
  white-space: pre-wrap;
  word-break: break-word;
  padding: 4px 8px 8px 8px;
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
