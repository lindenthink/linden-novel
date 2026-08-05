import { invoke } from '@tauri-apps/api/core';
import type { GenerateRequest, GenerateResponse, AiGenerationHistory } from '../types/ai_generation';

export async function aiGenerate(request: GenerateRequest): Promise<GenerateResponse> {
  return invoke('ai_generate', { request });
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
