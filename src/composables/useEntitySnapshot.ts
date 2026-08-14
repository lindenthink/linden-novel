import { ref } from "vue";
import {
  getEntityEvolution,
  listChapterSnapshots,
  listProjectEntities,
  generateChapterSnapshots,
  type EvolutionResponse,
  type SnapshotItem,
  type ProjectEntity,
} from "../api/entitySnapshot";
import { submitTask } from "../api/tasks";

const evolutionLoading = ref(false);
const currentEvolution = ref<EvolutionResponse | null>(null);

const chapterLoading = ref(false);
const chapterSnapshots = ref<SnapshotItem[]>([]);

const projectEntities = ref<ProjectEntity[]>([]);

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

  /** 加载项目内有快照的实体列表（去重） */
  async function fetchProjectEntities(projectId: string) {
    try {
      const res = await listProjectEntities(projectId);
      projectEntities.value = res.entities;
    } catch (e) {
      console.error("Failed to fetch project entities:", e);
      projectEntities.value = [];
    }
    return projectEntities.value;
  }

  /** 生成章节快照 */
  async function handleGenerateSnapshots(chapterId: string, projectId?: string) {
    generating.value = true;
    generateResult.value = null;
    try {
      const res = await generateChapterSnapshots(chapterId);
      generateResult.value = `成功 ${res.success_count} / 失败 ${res.failed_count}`;
      // 刷新章节快照
      await fetchChapterSnapshots(chapterId);
      // 刷新项目实体列表
      if (projectId) {
        await fetchProjectEntities(projectId);
      }
      return res;
    } finally {
      generating.value = false;
    }
  }

  /** 批量生成项目快照 — 改为提交异步任务（fire-and-forget） */
  async function handleBatchSnapshots(projectId: string) {
    generating.value = true;
    generateResult.value = "批量快照任务已提交，正在后台处理…";
    try {
      await submitTask({
        task_type: "generate_snapshots",
        project_id: projectId,
        target_type: null,
        target_id: null,
        content_hash: null,
        payload_json: {},
      });
      generateResult.value = "批量快照任务已提交，请在任务中心查看进度";
    } catch (e: any) {
      generateResult.value = e?.toString() ?? "提交失败";
      throw e;
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
    projectEntities,
    generating,
    generateResult,
    // actions
    fetchEvolution,
    fetchChapterSnapshots,
    fetchProjectEntities,
    handleGenerateSnapshots,
    handleBatchSnapshots,
    resetEvolution,
  };
}
