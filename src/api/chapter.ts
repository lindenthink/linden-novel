import { invoke } from "@tauri-apps/api/core";
import type { Chapter, CreateChapter, UpdateChapterMeta, ChapterContent } from "../types";

// ---- Chapter ----

export async function listChapters(volumeId: string): Promise<Chapter[]> {
  return invoke<Chapter[]>("list_chapters", { volumeId });
}

export async function getChapter(id: string): Promise<Chapter> {
  return invoke<Chapter>("get_chapter", { id });
}

export async function createChapter(input: CreateChapter): Promise<Chapter> {
  return invoke<Chapter>("create_chapter", { input });
}

export async function updateChapterMeta(id: string, input: UpdateChapterMeta): Promise<Chapter> {
  return invoke<Chapter>("update_chapter_meta", { id, input });
}

export async function deleteChapter(id: string): Promise<void> {
  return invoke("delete_chapter", { id });
}

export async function reorderChapters(chapterIds: string[]): Promise<void> {
  return invoke("reorder_chapters", { chapterIds });
}

// ---- Content ----

export async function getChapterContent(chapterId: string): Promise<ChapterContent> {
  return invoke<ChapterContent>("get_chapter_content", { chapterId });
}

/** 保存正文，返回权威 word_count */
export async function saveChapterContent(
  chapterId: string,
  contentJson: string,
  contentText: string,
): Promise<number> {
  return invoke<number>("save_chapter_content", { chapterId, contentJson, contentText });
}
