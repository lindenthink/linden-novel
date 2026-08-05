<script setup lang="ts">
import { watch, onBeforeUnmount } from "vue";
import { EditorContent, useEditor } from "@tiptap/vue-3";
import StarterKit from "@tiptap/starter-kit";
import Placeholder from "@tiptap/extension-placeholder";
import { useChapterStore } from "../../stores/chapter";
import BubbleToolbar from "./BubbleToolbar.vue";

const chapterStore = useChapterStore();

// TipTap 编辑器实例
const editor = useEditor({
  extensions: [
    StarterKit.configure({
      heading: { levels: [1, 2, 3] },
    }),
    Placeholder.configure({
      placeholder: "开始写作...",
    }),
  ],
  content: "",
  onUpdate: ({ editor }) => {
    // 内容变化时更新 store
    const json = editor.getJSON();
    const text = editor.getText();
    chapterStore.updateContent(JSON.stringify(json), text);
  },
});

// 监听 activeContent 变化，同步到编辑器
watch(
  () => chapterStore.activeContent,
  (content) => {
    if (!editor.value || !content) return;

    // 解析 JSON 内容并设置到编辑器
    try {
      const json = JSON.parse(content.content_json);
      editor.value.commands.setContent(json);
    } catch {
      // 如果 JSON 解析失败，使用纯文本
      editor.value.commands.setContent(content.content_text || "");
    }
  },
  { immediate: true }
);

// 自动保存（debounce 2 秒）
let saveTimer: ReturnType<typeof setTimeout> | null = null;
watch(
  () => chapterStore.dirty,
  (isDirty) => {
    if (!isDirty) return;

    // 清除之前的定时器
    if (saveTimer) clearTimeout(saveTimer);

    // 2 秒后自动保存
    saveTimer = setTimeout(async () => {
      await chapterStore.flushSave();
    }, 2000);
  }
);

onBeforeUnmount(() => {
  if (saveTimer) clearTimeout(saveTimer);
});
</script>

<template>
  <div class="linden-editor flex flex-col h-full">
    <!-- 气泡工具栏 -->
    <BubbleToolbar v-if="editor" :editor="editor" />

    <!-- 编辑器内容区 -->
    <div class="flex-1 overflow-auto">
      <EditorContent :editor="editor" class="prose prose-sm max-w-none px-8 py-6" />
    </div>
  </div>
</template>

<style scoped>
/* TipTap 编辑器基础样式 */
:deep(.tiptap) {
  outline: none;
  min-height: 100%;

  p {
    margin-bottom: 0.75em;
    line-height: 1.7;
  }

  h1, h2, h3 {
    margin-top: 1.5em;
    margin-bottom: 0.5em;
    font-weight: 600;
  }

  h1 { font-size: 1.875em; }
  h2 { font-size: 1.5em; }
  h3 { font-size: 1.25em; }

  ul, ol {
    padding-left: 1.5em;
    margin-bottom: 0.75em;
  }

  blockquote {
    border-left: 3px solid #d1d5db;
    padding-left: 1em;
    margin-left: 0;
    color: #6b7280;
  }

  code {
    background-color: #f3f4f6;
    padding: 0.2em 0.4em;
    border-radius: 3px;
    font-size: 0.9em;
  }

  pre {
    background-color: #1f2937;
    color: #f9fafb;
    padding: 1em;
    border-radius: 6px;
    overflow-x: auto;
    margin-bottom: 1em;

    code {
      background-color: transparent;
      padding: 0;
      color: inherit;
    }
  }

  /* Placeholder 样式 */
  p.is-editor-empty:first-child::before {
    content: attr(data-placeholder);
    float: left;
    color: #9ca3af;
    pointer-events: none;
    height: 0;
  }
}

/* 暗色模式适配 */
:deep(.dark .tiptap) {
  blockquote {
    border-left-color: #4b5563;
    color: #9ca3af;
  }

  code {
    background-color: #374151;
  }

  pre {
    background-color: #111827;
  }
}
</style>
