import { defineStore } from 'pinia';
import { ref } from 'vue';
import type { AiGenerationHistory, GenerateRequest, GenerateResponse } from '../types/ai_generation';
import * as aiGenerationApi from '../api/ai_generation';

export const useAiGenerationStore = defineStore('aiGeneration', () => {
  // State
  const history = ref<AiGenerationHistory[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);
  const currentGeneration = ref<GenerateResponse | null>(null);

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
  }

  return {
    // State
    history,
    loading,
    error,
    currentGeneration,
    // Actions
    generate,
    loadHistory,
    deleteHistory,
    clearHistory,
    clearCurrentGeneration,
  };
});
