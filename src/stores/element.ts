import { defineStore } from "pinia";
import { ref } from "vue";
import type {
  Character,
  CreateCharacter,
  UpdateCharacter,
  Storyline,
  CreateStoryline,
  UpdateStoryline,
  WorldviewEntry,
  CreateWorldviewEntry,
  UpdateWorldviewEntry,
  ChapterElement,
  CreateChapterElement,
  Foreshadow,
  CreateForeshadow,
  UpdateForeshadow,
} from "../types";
import * as api from "../api/element";

export const useElementStore = defineStore("element", () => {
  // ---- State ----
  const characters = ref<Character[]>([]);
  const storylines = ref<Storyline[]>([]);
  const worldview = ref<WorldviewEntry[]>([]);
  const foreshadows = ref<Foreshadow[]>([]);
  const chapterElements = ref<ChapterElement[]>([]);

  // ---- Character actions ----

  async function fetchCharacters(projectId: string) {
    characters.value = await api.listCharacters(projectId);
  }

  async function createCharacter(input: CreateCharacter) {
    const c = await api.createCharacter(input);
    characters.value.push(c);
    return c;
  }

  async function updateCharacter(id: string, input: UpdateCharacter) {
    const updated = await api.updateCharacter(id, input);
    const idx = characters.value.findIndex((c) => c.id === id);
    if (idx !== -1) characters.value[idx] = updated;
    return updated;
  }

  async function deleteCharacter(id: string) {
    await api.deleteCharacter(id);
    characters.value = characters.value.filter((c) => c.id !== id);
  }

  // ---- Storyline actions ----

  async function fetchStorylines(projectId: string) {
    storylines.value = await api.listStorylines(projectId);
  }

  async function createStoryline(input: CreateStoryline) {
    const s = await api.createStoryline(input);
    storylines.value.push(s);
    return s;
  }

  async function updateStoryline(id: string, input: UpdateStoryline) {
    const updated = await api.updateStoryline(id, input);
    const idx = storylines.value.findIndex((s) => s.id === id);
    if (idx !== -1) storylines.value[idx] = updated;
    return updated;
  }

  async function deleteStoryline(id: string) {
    await api.deleteStoryline(id);
    storylines.value = storylines.value.filter((s) => s.id !== id);
  }

  // ---- Worldview actions ----

  async function fetchWorldview(projectId: string) {
    worldview.value = await api.listWorldview(projectId);
  }

  async function createWorldview(input: CreateWorldviewEntry) {
    const w = await api.createWorldview(input);
    worldview.value.push(w);
    return w;
  }

  async function updateWorldview(id: string, input: UpdateWorldviewEntry) {
    const updated = await api.updateWorldview(id, input);
    const idx = worldview.value.findIndex((w) => w.id === id);
    if (idx !== -1) worldview.value[idx] = updated;
    return updated;
  }

  async function deleteWorldview(id: string) {
    await api.deleteWorldview(id);
    worldview.value = worldview.value.filter((w) => w.id !== id);
  }

  // ---- Foreshadow actions ----

  async function fetchForeshadows(projectId: string) {
    foreshadows.value = await api.listForeshadows(projectId);
  }

  async function createForeshadow(input: CreateForeshadow) {
    const f = await api.createForeshadow(input);
    foreshadows.value.push(f);
    return f;
  }

  async function updateForeshadow(id: string, input: UpdateForeshadow) {
    const updated = await api.updateForeshadow(id, input);
    const idx = foreshadows.value.findIndex((f) => f.id === id);
    if (idx !== -1) foreshadows.value[idx] = updated;
    return updated;
  }

  async function deleteForeshadow(id: string) {
    await api.deleteForeshadow(id);
    foreshadows.value = foreshadows.value.filter((f) => f.id !== id);
  }

  // ---- Chapter Element actions ----

  async function fetchChapterElements(chapterId: string) {
    chapterElements.value = await api.listChapterElements(chapterId);
  }

  async function addChapterElement(input: CreateChapterElement) {
    const ce = await api.addChapterElement(input);
    chapterElements.value.push(ce);
    return ce;
  }

  async function removeChapterElement(id: string) {
    await api.removeChapterElement(id);
    chapterElements.value = chapterElements.value.filter((ce) => ce.id !== id);
  }

  async function removeChapterElementByRef(
    chapterId: string,
    elementType: string,
    elementId: string,
  ) {
    await api.removeChapterElementByRef(chapterId, elementType, elementId);
    chapterElements.value = chapterElements.value.filter(
      (ce) => !(ce.element_type === elementType && ce.element_id === elementId),
    );
  }

  return {
    characters,
    storylines,
    worldview,
    foreshadows,
    chapterElements,
    fetchCharacters,
    createCharacter,
    updateCharacter,
    deleteCharacter,
    fetchStorylines,
    createStoryline,
    updateStoryline,
    deleteStoryline,
    fetchWorldview,
    createWorldview,
    updateWorldview,
    deleteWorldview,
    fetchForeshadows,
    createForeshadow,
    updateForeshadow,
    deleteForeshadow,
    fetchChapterElements,
    addChapterElement,
    removeChapterElement,
    removeChapterElementByRef,
  };
});
