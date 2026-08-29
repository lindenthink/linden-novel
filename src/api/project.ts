import { invoke } from "@tauri-apps/api/core";
import type { Project, CreateProject, UpdateProject } from "../types";
import type { Volume, CreateVolume, UpdateVolume } from "../types";

// ---- Project ----

export async function listProjects(): Promise<Project[]> {
  return invoke<Project[]>("list_projects");
}

export async function getProject(id: string): Promise<Project> {
  return invoke<Project>("get_project", { id });
}

export async function createProject(input: CreateProject): Promise<Project> {
  return invoke<Project>("create_project", { input });
}

export async function updateProject(id: string, input: UpdateProject): Promise<Project> {
  return invoke<Project>("update_project", { id, input });
}

export async function deleteProject(id: string): Promise<void> {
  return invoke("delete_project", { id });
}

/**
 * 将用户选择的本地图片复制到 app_data_dir/covers 下，返回相对路径（如 `covers/xxx.png`）。
 * 前端拿到后存入表单 cover_path，随 createProject/updateProject 一起保存到 DB。
 */
export async function saveProjectCover(filePath: string): Promise<string> {
  return invoke<string>("save_project_cover", { filePath });
}

// ---- Volume ----

export async function listVolumes(projectId: string): Promise<Volume[]> {
  return invoke<Volume[]>("list_volumes", { projectId });
}

export async function createVolume(input: CreateVolume): Promise<Volume> {
  return invoke<Volume>("create_volume", { input });
}

export async function updateVolume(id: string, input: UpdateVolume): Promise<Volume> {
  return invoke<Volume>("update_volume", { id, input });
}

export async function deleteVolume(id: string): Promise<void> {
  return invoke("delete_volume", { id });
}

export async function reorderVolumes(volumeIds: string[]): Promise<void> {
  return invoke("reorder_volumes", { volumeIds });
}
