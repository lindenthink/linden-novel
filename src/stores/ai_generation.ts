import { defineStore } from 'pinia';
import { ref } from 'vue';
import { listen } from '@tauri-apps/api/event';
import type {
  AiGenerationHistory,
  GenerateRequest,
  GenerateResponse,
  GenerationChunkEvent,
  GenerationDoneEvent,
  ReasoningChunkEvent,
} from '../types/ai_generation';
import * as aiGenerationApi from '../api/ai_generation';

export const useAiGenerationStore = defineStore('aiGeneration', () => {
  // State
  const history = ref<AiGenerationHistory[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);
  const currentGeneration = ref<GenerateResponse | null>(null);

  // 流式生成状态
  const streaming = ref(false);
  const streamContent = ref('');
  const streamError = ref<string | null>(null);
  // 推理过程状态（DeepSeek thinking 模式）
  const streamReasoning = ref('');
  const reasoningActive = ref(false);

  // Actions
  async function generate(request: GenerateRequest): Promise<GenerateResponse> {
    loading.value = true;
    error.value = null;
    try {
      const response = await aiGenerationApi.aiGenerate(request);
      currentGeneration.value = response;
      await loadHistory(request.chapter_id);
      return response;
    } catch (e: any) {
      error.value = e.message || '生成失败';
      throw e;
    } finally {
      loading.value = false;
    }
  }

  /// 流式生成：边收边显示，首 token 即更新 streamContent
  async function generateStream(request: GenerateRequest): Promise<void> {
    if (streaming.value) {
      throw new Error('已有流式生成正在进行中');
    }
    loading.value = true;
    error.value = null;
    streamError.value = null;
    streamContent.value = '';
    streamReasoning.value = '';
    reasoningActive.value = false;
    streaming.value = true;
    currentGeneration.value = null;

    // 监听推理过程事件（先于正文到达）
    const unlistenReasoning = await listen<ReasoningChunkEvent>(
      'ai-generation-reasoning',
      (event) => {
        const payload = event.payload;
        if (payload.reasoning) {
          streamReasoning.value += payload.reasoning;
          reasoningActive.value = true;
        }
        if (payload.done) {
          reasoningActive.value = false;
        }
      },
    );

    // 监听正文 chunk：首个正文 chunk 到达时关闭推理状态
    const unlistenChunk = await listen<GenerationChunkEvent>('ai-generation-chunk', (event) => {
      const payload = event.payload;
      if (payload.content) {
        streamContent.value += payload.content;
        reasoningActive.value = false;
      }
      if (payload.done) {
        streaming.value = false;
      }
    });

    const unlistenError = await listen<string>('ai-generation-error', (event) => {
      streamError.value = event.payload;
      streaming.value = false;
      reasoningActive.value = false;
    });

    const unlistenDone = await listen<GenerationDoneEvent>('ai-generation-done', (event) => {
      const payload = event.payload;
      streaming.value = false;
      reasoningActive.value = false;
      currentGeneration.value = {
        content: payload.content,
        history: payload.history,
      };
    });

    try {
      await aiGenerationApi.aiGenerateStream(request);
      // 流结束后刷新历史（new history 已通过 done 事件携带，但仍刷新确保顺序）
      await loadHistory(request.chapter_id);
    } catch (e: any) {
      error.value = e.message || '生成失败';
      streamError.value = e.message || '生成失败';
      throw e;
    } finally {
      streaming.value = false;
      reasoningActive.value = false;
      loading.value = false;
      unlistenReasoning();
      unlistenChunk();
      unlistenError();
      unlistenDone();
    }
  }

  async function loadHistory(chapterId: string): Promise<void> {
    loading.value = true;
    error.value = null;
    try {
      history.value = await aiGenerationApi.listAiGenerationHistory(chapterId);
    } catch (e: any) {
      error.value = e.message || '加载历史记录失败';
    } finally {
      loading.value = false;
    }
  }

  async function deleteHistory(id: string): Promise<void> {
    loading.value = true;
    error.value = null;
    try {
      await aiGenerationApi.deleteAiGenerationHistory(id);
      // 从本地列表中移除
      history.value = history.value.filter(h => h.id !== id);
    } catch (e: any) {
      error.value = e.message || '删除失败';
      throw e;
    } finally {
      loading.value = false;
    }
  }

  async function clearHistory(chapterId: string): Promise<void> {
    loading.value = true;
    error.value = null;
    try {
      await aiGenerationApi.deleteAiGenerationHistoryByChapter(chapterId);
      history.value = [];
    } catch (e: any) {
      error.value = e.message || '清空历史失败';
      throw e;
    } finally {
      loading.value = false;
    }
  }

  function clearCurrentGeneration(): void {
    currentGeneration.value = null;
    streamContent.value = '';
    streamReasoning.value = '';
    streamError.value = null;
    reasoningActive.value = false;
  }

  return {
    // State
    history,
    loading,
    error,
    currentGeneration,
    streaming,
    streamContent,
    streamError,
    streamReasoning,
    reasoningActive,
    // Actions
    generate,
    generateStream,
    loadHistory,
    deleteHistory,
    clearHistory,
    clearCurrentGeneration,
  };
});
