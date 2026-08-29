import { invoke } from "@tauri-apps/api/core";
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
  Inspiration,
  CreateInspiration,
  UpdateInspiration,
} from "../types";

// ---- Character ----

export async function listCharacters(projectId: string): Promise<Character[]> {
  return invoke<Character[]>("list_characters", { projectId });
}

export async function getCharacter(id: string): Promise<Character> {
  return invoke<Character>("get_character", { id });
}

export async function createCharacter(input: CreateCharacter): Promise<Character> {
  return invoke<Character>("create_character", { input });
}

export async function updateCharacter(id: string, input: UpdateCharacter): Promise<Character> {
  return invoke<Character>("update_character", { id, input });
}

export async function deleteCharacter(id: string): Promise<void> {
  return invoke("delete_character", { id });
}

// ---- Storyline ----

export async function listStorylines(projectId: string): Promise<Storyline[]> {
  return invoke<Storyline[]>("list_storylines", { projectId });
}

export async function getStoryline(id: string): Promise<Storyline> {
  return invoke<Storyline>("get_storyline", { id });
}

export async function createStoryline(input: CreateStoryline): Promise<Storyline> {
  return invoke<Storyline>("create_storyline", { input });
}

export async function updateStoryline(id: string, input: UpdateStoryline): Promise<Storyline> {
  return invoke<Storyline>("update_storyline", { id, input });
}

export async function deleteStoryline(id: string): Promise<void> {
  return invoke("delete_storyline", { id });
}

// ---- Worldview ----

export async function listWorldview(projectId: string): Promise<WorldviewEntry[]> {
  return invoke<WorldviewEntry[]>("list_worldview", { projectId });
}

export async function getWorldview(id: string): Promise<WorldviewEntry> {
  return invoke<WorldviewEntry>("get_worldview", { id });
}

export async function createWorldview(input: CreateWorldviewEntry): Promise<WorldviewEntry> {
  return invoke<WorldviewEntry>("create_worldview", { input });
}

export async function updateWorldview(id: string, input: UpdateWorldviewEntry): Promise<WorldviewEntry> {
  return invoke<WorldviewEntry>("update_worldview", { id, input });
}

export async function deleteWorldview(id: string): Promise<void> {
  return invoke("delete_worldview", { id });
}

// ---- Chapter Element ----

export async function listChapterElements(chapterId: string): Promise<ChapterElement[]> {
  return invoke<ChapterElement[]>("list_chapter_elements", { chapterId });
}

export async function addChapterElement(input: CreateChapterElement): Promise<ChapterElement> {
  return invoke<ChapterElement>("add_chapter_element", { input });
}

export async function removeChapterElement(id: string): Promise<void> {
  return invoke("remove_chapter_element", { id });
}

export async function removeChapterElementByRef(
  chapterId: string,
  elementType: string,
  elementId: string,
): Promise<void> {
  return invoke("remove_chapter_element_by_ref", { chapterId, elementType, elementId });
}

// ---- Foreshadow ----

export async function listForeshadows(projectId: string): Promise<Foreshadow[]> {
  return invoke<Foreshadow[]>("list_foreshadows", { projectId });
}

export async function getForeshadow(id: string): Promise<Foreshadow> {
  return invoke<Foreshadow>("get_foreshadow", { id });
}

export async function createForeshadow(input: CreateForeshadow): Promise<Foreshadow> {
  return invoke<Foreshadow>("create_foreshadow", { input });
}

export async function updateForeshadow(id: string, input: UpdateForeshadow): Promise<Foreshadow> {
  return invoke<Foreshadow>("update_foreshadow", { id, input });
}

export async function deleteForeshadow(id: string): Promise<void> {
  return invoke("delete_foreshadow", { id });
}

// ---- Inspiration ----

export async function listInspirations(projectId: string): Promise<Inspiration[]> {
  return invoke<Inspiration[]>("list_inspirations", { projectId });
}

export async function getInspiration(id: string): Promise<Inspiration> {
  return invoke<Inspiration>("get_inspiration", { id });
}

export async function createInspiration(input: CreateInspiration): Promise<Inspiration> {
  return invoke<Inspiration>("create_inspiration", { input });
}

export async function updateInspiration(id: string, input: UpdateInspiration): Promise<Inspiration> {
  return invoke<Inspiration>("update_inspiration", { id, input });
}

export async function deleteInspiration(id: string): Promise<void> {
  return invoke("delete_inspiration", { id });
}
