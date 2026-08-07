import { ref } from "vue";
import {
  generateChapterSummary,
  batchGenerateSummaries,
  syncProjectEmbeddings,
  getChapterSummary,
  type GenerateSummaryResponse,
  type BatchSummaryResponse,
  type SyncEmbeddingsResponse,
} from "../api/longContext";

/**
 * SP4: 长上下文引擎前端状态与操作
 */
const summaryLoading = ref(false);
const summaryError = ref<string | null>(null);

const batchProgress = ref<string | null>(null);
const batchLoading = ref(false);

const embeddingLoading = ref(false);
const embeddingStatus = ref<string | null>(null);

export function useLongContext() {
  /** 为指定章节生成摘要 */
  async function handleGenerateSummary(
    chapterId: string,
    force = false,
    onSuccess?: (res: GenerateSummaryResponse) => void
  ) {
    summaryLoading.value = true;
    summaryError.value = null;
    try {
      const res = await generateChapterSummary(chapterId, force);
      onSuccess?.(res);
      return res;
    } catch (e: any) {
      summaryError.value = e?.toString() ?? "Unknown error";
      throw e;
    } finally {
      summaryLoading.value = false;
    }
  }

  /** 批量生成项目摘要 */
  async function handleBatchSummaries(
    projectId: string,
    onSuccess?: (res: BatchSummaryResponse) => void
  ) {
    batchLoading.value = true;
    batchProgress.value = null;
    try {
      const res = await batchGenerateSummaries(projectId);
      batchProgress.value = `成功 ${res.success_count} / 失败 ${res.failed_count}`;
      onSuccess?.(res);
      return res;
    } finally {
      batchLoading.value = false;
    }
  }

  /** 同步项目嵌入 */
  async function handleSyncEmbeddings(
    projectId: string,
    onSuccess?: (res: SyncEmbeddingsResponse) => void
  ) {
    embeddingLoading.value = true;
    embeddingStatus.value = null;
    try {
      const res = await syncProjectEmbeddings(projectId);
      embeddingStatus.value = `已嵌入 ${res.embedded_count} 个条目`;
      onSuccess?.(res);
      return res;
    } finally {
      embeddingLoading.value = false;
    }
  }

  /** 读取章节摘要 */
  async function fetchSummary(chapterId: string) {
    return getChapterSummary(chapterId);
  }

  function resetError() {
    summaryError.value = null;
    batchProgress.value = null;
    embeddingStatus.value = null;
  }

  return {
    // state
    summaryLoading,
    summaryError,
    batchLoading,
    batchProgress,
    embeddingLoading,
    embeddingStatus,
    // actions
    handleGenerateSummary,
    handleBatchSummaries,
    handleSyncEmbeddings,
    fetchSummary,
    resetError,
  };
}
