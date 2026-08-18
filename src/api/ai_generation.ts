import { invoke } from '@tauri-apps/api/core';
import type { GenerateRequest, GenerateResponse, AiGenerationHistory } from '../types/ai_generation';

export async function aiGenerate(request: GenerateRequest): Promise<GenerateResponse> {
  return invoke('ai_generate', { request });
}

/// 流式生成：不返回内容（通过事件推送），仅返回调用是否成功
export async function aiGenerateStream(request: GenerateRequest): Promise<void> {
  return invoke('ai_generate_stream', { request });
}

export async function listAiGenerationHistory(chapterId: string): Promise<AiGenerationHistory[]> {
  return invoke('list_ai_generation_history', { chapterId });
}

export async function getAiGenerationHistory(id: string): Promise<AiGenerationHistory> {
  return invoke('get_ai_generation_history', { id });
}

export async function deleteAiGenerationHistory(id: string): Promise<void> {
  return invoke('delete_ai_generation_history', { id });
}

export async function deleteAiGenerationHistoryByChapter(chapterId: string): Promise<number> {
  return invoke('delete_ai_generation_history_by_chapter', { chapterId });
}
