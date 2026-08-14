import { ref } from "vue";
import {
  generateChapterSummary,
  getChapterSummary,
  type GenerateSummaryResponse,
} from "../api/longContext";
import { submitTask } from "../api/tasks";
import { useTaskCenter } from "./useTaskCenter";

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
  // 注意：useTaskCenter 是模块级单例，不在此解构使用，实际状态通过 TaskCenter UI 展示
  useTaskCenter();

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

  /** 批量生成项目摘要 — 改为提交异步任务（fire-and-forget） */
  async function handleBatchSummaries(
    projectId: string,
    onSuccess?: () => void
  ) {
    batchLoading.value = true;
    batchProgress.value = "批量摘要任务已提交，正在后台处理…";
    try {
      await submitTask({
        task_type: "generate_summary",
        project_id: projectId,
        target_type: null,
        target_id: null,
        content_hash: null,
        payload_json: {},
      });
      onSuccess?.();
    } catch (e: any) {
      batchProgress.value = e?.toString() ?? "提交失败";
      throw e;
    } finally {
      batchLoading.value = false;
    }
  }

  /** 同步项目嵌入 — 改为提交异步任务（fire-and-forget） */
  async function handleSyncEmbeddings(
    projectId: string,
    onSuccess?: () => void
  ) {
    embeddingLoading.value = true;
    embeddingStatus.value = "同步任务已提交，正在后台处理…";
    try {
      // 构造一个固定的 payload + content_hash（让 sync_embeddings 任务幂等：
      // 项目级全量同步总是会跑完整流程，hash 使用 project_id 即可）
      // 但更好的做法：sync_embeddings 不走 content_hash 幂等（因为每次调用意图都是全量）
      // 所以不传 content_hash，允许重复提交。
      await submitTask({
        task_type: "sync_embeddings",
        project_id: projectId,
        target_type: null,
        target_id: null,
        content_hash: null,
        payload_json: {}, // app_data_dir 在后端从 app handle 获取
      });
      onSuccess?.();
    } catch (e: any) {
      embeddingStatus.value = e?.toString() ?? "提交失败";
      throw e;
    } finally {
      // 注意：因为是 fire-and-forget，loading 很快置为 false
      // 真实进度通过 TaskCenter 的事件实时更新
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
