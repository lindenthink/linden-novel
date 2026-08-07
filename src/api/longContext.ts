import { invoke } from "@tauri-apps/api/core";

export interface GenerateSummaryResponse {
  chapter_id: string;
  summary: string;
  char_count: number;
}

export interface BatchSummaryResponse {
  project_id: string;
  success_count: number;
  failed_count: number;
}

export interface SyncEmbeddingsResponse {
  project_id: string;
  embedded_count: number;
}

export interface RagSearchItem {
  source_type: string;
  source_id: string;
  title: string;
  content: string;
  score: number;
}

export interface RagSearchResponse {
  results: RagSearchItem[];
}

/** 为指定章节生成摘要 */
export async function generateChapterSummary(
  chapterId: string,
  force = false
): Promise<GenerateSummaryResponse> {
  return invoke<GenerateSummaryResponse>("generate_chapter_summary", {
    request: { chapter_id: chapterId, force },
  });
}

/** 批量为项目内所有无摘要章节生成摘要 */
export async function batchGenerateSummaries(
  projectId: string
): Promise<BatchSummaryResponse> {
  return invoke<BatchSummaryResponse>("batch_generate_summaries", {
    request: { project_id: projectId },
  });
}

/** 为项目所有元素同步嵌入 */
export async function syncProjectEmbeddings(
  projectId: string
): Promise<SyncEmbeddingsResponse> {
  return invoke<SyncEmbeddingsResponse>("sync_project_embeddings", {
    request: { project_id: projectId },
  });
}

/** 执行 RAG 语义检索 */
export async function ragSearch(
  projectId: string,
  query: string,
  options?: {
    topK?: number;
    minScore?: number;
    excludeChapterId?: string;
  }
): Promise<RagSearchResponse> {
  return invoke<RagSearchResponse>("rag_search", {
    request: {
      project_id: projectId,
      query,
      top_k: options?.topK ?? 3,
      min_score: options?.minScore ?? 0.3,
      exclude_chapter_id: options?.excludeChapterId ?? null,
    },
  });
}

/** 删除项目的全部嵌入 */
export async function deleteProjectEmbeddings(
  projectId: string
): Promise<void> {
  return invoke("delete_project_embeddings", {
    request: { project_id: projectId },
  });
}

/** 获取章节摘要 */
export async function getChapterSummary(
  chapterId: string
): Promise<string | null> {
  return invoke<string | null>("get_chapter_summary", {
    chapter_id: chapterId,
  });
}
