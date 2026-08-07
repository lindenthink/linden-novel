import { invoke } from "@tauri-apps/api/core";

export interface SnapshotWithChapter {
  id: string;
  chapter_id: string;
  chapter_title: string;
  order_index: number;
  state_json: string;
  summary: string;
  changes: string | null;
  created_at: string;
}

export interface EvolutionResponse {
  entity_id: string;
  entity_type: string;
  name: string;
  snapshots: SnapshotWithChapter[];
}

export interface SnapshotItem {
  id: string;
  entity_type: string;
  entity_id: string;
  summary: string;
  state_json: string;
  changes: string | null;
}

export interface ListChapterSnapshotsResponse {
  snapshots: SnapshotItem[];
}

export interface GenerateSnapshotsResponse {
  chapter_id: string;
  success_count: number;
  failed_count: number;
}

export interface BatchSnapshotsResponse {
  project_id: string;
  success_count: number;
  failed_count: number;
}

/** 为指定章节生成实体快照 */
export async function generateChapterSnapshots(
  chapterId: string
): Promise<GenerateSnapshotsResponse> {
  return invoke<GenerateSnapshotsResponse>("generate_chapter_snapshots", {
    request: { chapter_id: chapterId },
  });
}

/** 批量生成项目内所有章节的实体快照 */
export async function batchGenerateSnapshots(
  projectId: string
): Promise<BatchSnapshotsResponse> {
  return invoke<BatchSnapshotsResponse>("batch_generate_snapshots", {
    request: { project_id: projectId },
  });
}

/** 获取实体的演变历史 */
export async function getEntityEvolution(
  entityType: string,
  entityId: string
): Promise<EvolutionResponse> {
  return invoke<EvolutionResponse>("get_entity_evolution", {
    request: { entity_type: entityType, entity_id: entityId },
  });
}

/** 获取指定章节的全部实体快照 */
export async function listChapterSnapshots(
  chapterId: string
): Promise<ListChapterSnapshotsResponse> {
  return invoke<ListChapterSnapshotsResponse>("list_chapter_snapshots", {
    request: { chapter_id: chapterId },
  });
}

/** 删除项目的全部实体快照 */
export async function deleteProjectSnapshots(
  projectId: string
): Promise<void> {
  return invoke("delete_project_snapshots", {
    request: { project_id: projectId },
  });
}
