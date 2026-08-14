import { invoke } from "@tauri-apps/api/core";

/** 任务状态 */
export type TaskStatus =
  | "pending"
  | "running"
  | "completed"
  | "failed"
  | "cancelled";

/** 任务类型 */
export type TaskType =
  | "embed_element"
  | "embed_chapter"
  | "sync_embeddings"
  | "generate_summary"
  | "generate_snapshots";

/** 异步任务实体（与后端 AsyncTask 一一对应） */
export interface AsyncTask {
  id: string;
  task_type: TaskType;
  project_id: string;
  target_type: string | null;
  target_id: string | null;
  content_hash: string | null;
  payload_json: string | null;

  status: TaskStatus;
  progress_current: number;
  progress_total: number;

  result_json: string | null;
  error_message: string | null;

  created_at: string;
  started_at: string | null;
  completed_at: string | null;
}

/** 提交新任务的请求 */
export interface NewTaskRequest {
  task_type: TaskType;
  project_id: string;
  target_type?: string | null;
  target_id?: string | null;
  content_hash?: string | null;
  payload_json?: Record<string, unknown> | null;
}

/** Tauri 事件 payload：任务进度更新 */
export interface TaskProgressEvent {
  task_id: string;
  status?: TaskStatus;
  progress_current: number;
  progress_total: number;
}

/** Tauri 事件 payload：任务完成 */
export interface TaskCompletedEvent {
  task_id: string;
  status: TaskStatus;
  result?: unknown;
  error?: string;
}

/** 提交一个新任务（幂等） */
export async function submitTask(
  input: NewTaskRequest
): Promise<AsyncTask> {
  return invoke<AsyncTask>("submit_task", { input });
}

/** 查询项目下的任务列表 */
export async function listTasks(
  projectId: string,
  statusFilter?: TaskStatus
): Promise<AsyncTask[]> {
  return invoke<AsyncTask[]>("list_tasks", {
    projectId,
    statusFilter: statusFilter ?? null,
  });
}

/** 查询单个任务 */
export async function getTask(taskId: string): Promise<AsyncTask> {
  return invoke<AsyncTask>("get_task", { taskId });
}

/** 取消任务（仅 pending/running 可取消） */
export async function cancelTask(taskId: string): Promise<void> {
  return invoke("cancel_task", { taskId });
}
