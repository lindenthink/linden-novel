import { defineStore } from "pinia";
import { ref } from "vue";
import type { Project, Volume } from "../types";
import * as api from "../api/project";

export const useProjectStore = defineStore("project", () => {
  // ---- State ----
  const projects = ref<Project[]>([]);
  const currentProject = ref<Project | null>(null);
  const volumes = ref<Volume[]>([]);
  const loading = ref(false);

  // ---- Project actions ----

  async function fetchProjects() {
    loading.value = true;
    try {
      projects.value = await api.listProjects();
    } finally {
      loading.value = false;
    }
  }

  async function selectProject(id: string) {
    currentProject.value = await api.getProject(id);
    volumes.value = await api.listVolumes(id);
  }

  async function createProject(title: string) {
    const p = await api.createProject({ title });
    projects.value.unshift(p);
    return p;
  }

  async function updateProject(id: string, input: Parameters<typeof api.updateProject>[1]) {
    const updated = await api.updateProject(id, input);
    if (currentProject.value?.id === id) {
      currentProject.value = updated;
    }
    const idx = projects.value.findIndex((p) => p.id === id);
    if (idx !== -1) projects.value[idx] = updated;
    return updated;
  }

  async function deleteProject(id: string) {
    await api.deleteProject(id);
    projects.value = projects.value.filter((p) => p.id !== id);
    if (currentProject.value?.id === id) {
      currentProject.value = null;
      volumes.value = [];
    }
  }

  // ---- Volume actions ----

  async function createVolume(title: string) {
    if (!currentProject.value) return;
    const v = await api.createVolume({
      project_id: currentProject.value.id,
      title,
    });
    volumes.value.push(v);
    return v;
  }

  async function updateVolume(id: string, input: Parameters<typeof api.updateVolume>[1]) {
    const updated = await api.updateVolume(id, input);
    const idx = volumes.value.findIndex((v) => v.id === id);
    if (idx !== -1) volumes.value[idx] = updated;
    return updated;
  }

  async function deleteVolume(id: string) {
    await api.deleteVolume(id);
    volumes.value = volumes.value.filter((v) => v.id !== id);
  }

  async function reorderVolumes(ids: string[]) {
    await api.reorderVolumes(ids);
    // 按新顺序重排本地
    const map = new Map(volumes.value.map((v) => [v.id, v]));
    volumes.value = ids.map((id, i) => {
      const v = map.get(id)!;
      return { ...v, order_index: i };
    });
  }

  return {
    projects,
    currentProject,
    volumes,
    loading,
    fetchProjects,
    selectProject,
    createProject,
    updateProject,
    deleteProject,
    createVolume,
    updateVolume,
    deleteVolume,
    reorderVolumes,
  };
});
