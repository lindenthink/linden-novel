import { invoke } from "@tauri-apps/api/core";

export async function exportProject(
  projectId: string,
  format: string,
  path: string
): Promise<void> {
  return invoke("export_project", { projectId, format, path });
}

export async function importProject(path: string): Promise<string> {
  return invoke<string>("import_project", { path });
}
