import { ref } from "vue";
import {
  getEntityEvolution,
  listChapterSnapshots,
  generateChapterSnapshots,
  batchGenerateSnapshots,
  type EvolutionResponse,
  type SnapshotItem,
} from "../api/entitySnapshot";

const evolutionLoading = ref(false);
const currentEvolution = ref<EvolutionResponse | null>(null);

const chapterLoading = ref(false);
const chapterSnapshots = ref<SnapshotItem[]>([]);

const generating = ref(false);
const generateResult = ref<string | null>(null);

export function useEntitySnapshot() {
  /** 加载实体演变历史 */
  async function fetchEvolution(entityType: string, entityId: string) {
    evolutionLoading.value = true;
    currentEvolution.value = null;
    try {
      currentEvolution.value = await getEntityEvolution(entityType, entityId);
    } finally {
      evolutionLoading.value = false;
    }
    return currentEvolution.value;
  }

  /** 加载章节快照 */
  async function fetchChapterSnapshots(chapterId: string) {
    chapterLoading.value = true;
    chapterSnapshots.value = [];
    try {
      const res = await listChapterSnapshots(chapterId);
      chapterSnapshots.value = res.snapshots;
    } finally {
      chapterLoading.value = false;
    }
    return chapterSnapshots.value;
  }

  /** 生成章节快照 */
  async function handleGenerateSnapshots(chapterId: string) {
    generating.value = true;
    generateResult.value = null;
    try {
      const res = await generateChapterSnapshots(chapterId);
      generateResult.value = `成功 ${res.success_count} / 失败 ${res.failed_count}`;
      // 刷新章节快照
      await fetchChapterSnapshots(chapterId);
      return res;
    } finally {
      generating.value = false;
    }
  }

  /** 批量生成项目快照 */
  async function handleBatchSnapshots(projectId: string) {
    generating.value = true;
    generateResult.value = null;
    try {
      const res = await batchGenerateSnapshots(projectId);
      generateResult.value = `成功 ${res.success_count} / 失败 ${res.failed_count}`;
      return res;
    } finally {
      generating.value = false;
    }
  }

  function resetEvolution() {
    currentEvolution.value = null;
  }

  return {
    // state
    evolutionLoading,
    currentEvolution,
    chapterLoading,
    chapterSnapshots,
    generating,
    generateResult,
    // actions
    fetchEvolution,
    fetchChapterSnapshots,
    handleGenerateSnapshots,
    handleBatchSnapshots,
    resetEvolution,
  };
}
