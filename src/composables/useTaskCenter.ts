import { computed, ref, onScopeDispose, shallowRef } from "vue";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import {
  listTasks,
  cancelTask as apiCancelTask,
  getTask,
  type AsyncTask,
  type TaskCompletedEvent,
  type TaskProgressEvent,
  type TaskStatus,
  type TaskType,
} from "../api/tasks";

/**
 * 任务中心状态管理（模块级单例）
 *
 * 多个组件调用 useTaskCenter() 会共享同一份状态。
 * 必须在进入项目时调用 init(projectId) 拉取历史任务并启动事件监听，
 * 离开项目时调用 cleanup()。
 */
// ---- 模块级共享状态 ----
const tasks = ref<AsyncTask[]>([]);
const loading = ref(false);
const error = ref<string | null>(null);
const currentProjectId = ref<string | null>(null);

// 事件监听器句柄
const unlistenProgress = shallowRef<UnlistenFn | null>(null);
const unlistenCompleted = shallowRef<UnlistenFn | null>(null);
const unlistenCreated = shallowRef<UnlistenFn | null>(null);

export function useTaskCenter() {
  // ---- 衍生状态 ----

  /** 活跃任务（pending + running） */
  const activeTasks = computed(() =>
    tasks.value.filter(
      (t) => t.status === "pending" || t.status === "running"
    )
  );

  /** 活跃任务数量（用于角标） */
  const activeCount = computed(() => activeTasks.value.length);

  /** 最近完成任务（completed/failed/cancelled，最多 50 条） */
  const recentTasks = computed(() =>
    tasks.value
      .filter(
        (t) =>
          t.status === "completed" ||
          t.status === "failed" ||
          t.status === "cancelled"
      )
      .slice(0, 50)
  );

  // ---- 内部工具 ----

  function upsertTask(task: AsyncTask) {
    const idx = tasks.value.findIndex((t) => t.id === task.id);
    if (idx >= 0) {
      tasks.value[idx] = task;
    } else {
      tasks.value.unshift(task);
    }
  }

  function patchTask(id: string, patch: Partial<AsyncTask>) {
    const idx = tasks.value.findIndex((t) => t.id === id);
    if (idx >= 0) {
      tasks.value[idx] = { ...tasks.value[idx], ...patch };
    }
  }

  // ---- 事件监听 ----

  async function startListening() {
    if (unlistenProgress.value) return; // 已启动

    // 监听新任务创建
    unlistenCreated.value = await listen(
      "task-created",
      async (e) => {
        const p = e.payload as { task_id: string; project_id: string };
        // 只处理当前项目的任务
        if (currentProjectId.value && p.project_id === currentProjectId.value) {
          // 拉取完整任务数据
          try {
            const task = await getTask(p.task_id);
            if (task) {
              upsertTask(task);
            }
          } catch {
            // 忽略错误，任务可能稍后通过刷新获取
          }
        }
      }
    );

    unlistenProgress.value = await listen<TaskProgressEvent>(
      "task-progress",
      (e) => {
        const p = e.payload;
        patchTask(p.task_id, {
          status: p.status ?? "running",
          progress_current: p.progress_current,
          progress_total: p.progress_total,
        });
      }
    );

    unlistenCompleted.value = await listen<TaskCompletedEvent>(
      "task-completed",
      (e) => {
        const p = e.payload;
        // 收到完成事件后，从后端拉取最新数据以保证字段完整
        getTask(p.task_id)
          .then((task) => upsertTask(task))
          .catch(() => {
            // 拉取失败时退化到事件数据
            patchTask(p.task_id, {
              status: p.status,
              error_message: p.error ?? null,
            });
          });
      }
    );
  }

  function stopListening() {
    unlistenCreated.value?.();
    unlistenProgress.value?.();
    unlistenCompleted.value?.();
    unlistenCreated.value = null;
    unlistenProgress.value = null;
    unlistenCompleted.value = null;
  }

  // ---- 对外动作 ----

  /** 进入项目时初始化：拉取历史任务 + 启动事件监听 */
  async function init(projectId: string) {
    currentProjectId.value = projectId;
    loading.value = true;
    error.value = null;
    try {
      tasks.value = await listTasks(projectId);
      await startListening();
    } catch (e: any) {
      error.value = e?.toString() ?? "Failed to load tasks";
    } finally {
      loading.value = false;
    }
  }

  /** 离开项目时清理 */
  function cleanup() {
    stopListening();
    tasks.value = [];
    currentProjectId.value = null;
  }

  /** 刷新任务列表 */
  async function refresh(projectId?: string) {
    const pid = projectId ?? currentProjectId.value;
    if (!pid) return;
    try {
      tasks.value = await listTasks(pid);
    } catch (e: any) {
      error.value = e?.toString() ?? "Failed to refresh";
    }
  }

  /** 取消任务 */
  async function cancelTask(taskId: string) {
    try {
      await apiCancelTask(taskId);
      patchTask(taskId, { status: "cancelled" });
    } catch (e: any) {
      error.value = e?.toString() ?? "Failed to cancel task";
      throw e;
    }
  }

  // 自动清理（如果在 effect scope 内使用，但模块级监听不会被释放）
  onScopeDispose(() => {
    // 注意：这里不调用 stopListening，因为监听是全局共享的
    // 由 EditorView 的 unmount 显式调用 cleanup()
  });

  return {
    // state
    tasks,
    loading,
    error,
    currentProjectId,
    // derived
    activeTasks,
    activeCount,
    recentTasks,
    // actions
    init,
    cleanup,
    refresh,
    cancelTask,
  };
}

// ---- 共享工具：任务类型/状态文案 ----

const TASK_TYPE_LABELS: Record<TaskType, string> = {
  embed_element: "元素嵌入",
  embed_chapter: "章节切片嵌入",
  sync_embeddings: "同步嵌入",
  generate_summary: "生成摘要",
  generate_snapshots: "生成实体快照",
};

const TASK_STATUS_LABELS: Record<TaskStatus, string> = {
  pending: "等待中",
  running: "进行中",
  completed: "已完成",
  failed: "失败",
  cancelled: "已取消",
};

/** 任务类型中文标签 */
export function taskTypeLabel(t: TaskType): string {
  return TASK_TYPE_LABELS[t] ?? t;
}

/** 任务状态中文标签 */
export function taskStatusLabel(s: TaskStatus): string {
  return TASK_STATUS_LABELS[s] ?? s;
}

/** 任务状态对应的 naive-ui Tag 类型 */
export function taskStatusTagType(
  s: TaskStatus
): "default" | "info" | "success" | "error" | "warning" {
  switch (s) {
    case "pending":
      return "default";
    case "running":
      return "info";
    case "completed":
      return "success";
    case "failed":
      return "error";
    case "cancelled":
      return "warning";
  }
}
