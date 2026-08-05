import { defineStore } from "pinia";
import { ref } from "vue";
import type { Chapter, ChapterContent } from "../types";
import * as api from "../api/chapter";

export const useChapterStore = defineStore("chapter", () => {
  // ---- State ----
  const chapters = ref<Chapter[]>([]);
  const activeChapterId = ref<string | null>(null);
  const activeContent = ref<ChapterContent | null>(null);
  /** 正文是否有未保存修改 */
  const dirty = ref(false);
  /** 正在保存中 */
  const saving = ref(false);
  /** 当前活跃章的权威字数（由 save 返回） */
  const wordCount = ref(0);

  // ---- Chapter list actions ----

  async function fetchChapters(volumeId: string) {
    chapters.value = await api.listChapters(volumeId);
  }

  async function createChapter(volumeId: string, projectId: string, title: string) {
    const ch = await api.createChapter({ volume_id: volumeId, project_id: projectId, title });
    chapters.value.push(ch);
    return ch;
  }

  async function updateChapterMeta(id: string, input: Parameters<typeof api.updateChapterMeta>[1]) {
    const updated = await api.updateChapterMeta(id, input);
    const idx = chapters.value.findIndex((c) => c.id === id);
    if (idx !== -1) chapters.value[idx] = updated;
    return updated;
  }

  async function deleteChapter(id: string) {
    await api.deleteChapter(id);
    chapters.value = chapters.value.filter((c) => c.id !== id);
    if (activeChapterId.value === id) {
      activeChapterId.value = null;
      activeContent.value = null;
      dirty.value = false;
      wordCount.value = 0;
    }
  }

  async function reorderChapters(ids: string[]) {
    await api.reorderChapters(ids);
    const map = new Map(chapters.value.map((c) => [c.id, c]));
    chapters.value = ids.map((id, i) => {
      const c = map.get(id)!;
      return { ...c, order_index: i };
    });
  }

  // ---- Active chapter + content ----

  /** 切换活跃章：先 flush 当前脏数据，再加载新章 */
  async function setActiveChapter(chapterId: string) {
    if (dirty.value && activeChapterId.value) {
      await flushSave();
    }
    activeChapterId.value = chapterId;
    const ch = chapters.value.find((c) => c.id === chapterId);
    wordCount.value = ch?.word_count ?? 0;
    activeContent.value = await api.getChapterContent(chapterId);
    dirty.value = false;
  }

  /** 标记正文已修改 */
  function markDirty() {
    dirty.value = true;
  }

  /** 保存正文（debounce 后调用），返回权威 word_count */
  async function flushSave(): Promise<number | null> {
    if (!dirty.value || !activeChapterId.value || !activeContent.value) return null;
    saving.value = true;
    try {
      const wc = await api.saveChapterContent(
        activeChapterId.value,
        activeContent.value.content_json,
        activeContent.value.content_text,
      );
      wordCount.value = wc;
      dirty.value = false;
      // 更新 chapters 列表中的 word_count
      const idx = chapters.value.findIndex((c) => c.id === activeChapterId.value);
      if (idx !== -1) chapters.value[idx].word_count = wc;
      return wc;
    } finally {
      saving.value = false;
    }
  }

  /** 更新内存中的正文（编辑器 onUpdate 时调用） */
  function updateContent(contentJson: string, contentText: string) {
    if (activeContent.value) {
      activeContent.value.content_json = contentJson;
      activeContent.value.content_text = contentText;
    }
    dirty.value = true;
  }

  return {
    chapters,
    activeChapterId,
    activeContent,
    dirty,
    saving,
    wordCount,
    fetchChapters,
    createChapter,
    updateChapterMeta,
    deleteChapter,
    reorderChapters,
    setActiveChapter,
    markDirty,
    flushSave,
    updateContent,
  };
});
