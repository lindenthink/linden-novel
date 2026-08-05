<script setup lang="ts">
import { BubbleMenu } from "@tiptap/vue-3/menus";
import type { Editor } from "@tiptap/vue-3";

const props = defineProps<{
  editor: Editor;
}>();

const emit = defineEmits<{
  (e: "ai-continue"): void;
  (e: "ai-rewrite"): void;
  (e: "ai-expand"): void;
  (e: "ai-polish"): void;
}>();

function isActive(name: string, attrs?: Record<string, any>) {
  return props.editor.isActive(name, attrs);
}

function toggle(name: string, attrs?: Record<string, any>) {
  props.editor.chain().focus().toggleMark(name, attrs).run();
}

function toggleHeading(level: 1 | 2 | 3) {
  props.editor.chain().focus().toggleHeading({ level }).run();
}

function toggleList(type: "bulletList" | "orderedList") {
  const itemType = type === "bulletList" ? "listItem" : "listItem";
  props.editor.chain().focus().toggleList(type, itemType).run();
}

function toggleBlockquote() {
  props.editor.chain().focus().toggleBlockquote().run();
}

function insertSceneBreak() {
  props.editor.chain().focus().setSceneBreak().run();
}

function handleAIContinue() {
  emit("ai-continue");
}

function handleAIRewrite() {
  emit("ai-rewrite");
}

function handleAIExpand() {
  emit("ai-expand");
}

function handleAIPolish() {
  emit("ai-polish");
}
</script>

<template>
  <BubbleMenu :editor="editor" :options="{ placement: 'top', offset: 8 }">
    <div class="bubble-toolbar flex items-center gap-1 bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700 px-2 py-1">
      <!-- 标题 -->
      <button
        class="px-2 py-1 rounded text-xs font-medium transition-colors"
        :class="isActive('heading', { level: 1 }) ? 'bg-linden-primary/20 text-linden-primary' : 'hover:bg-gray-100 dark:hover:bg-gray-700'"
        @click="toggleHeading(1)"
      >
        H1
      </button>
      <button
        class="px-2 py-1 rounded text-xs font-medium transition-colors"
        :class="isActive('heading', { level: 2 }) ? 'bg-linden-primary/20 text-linden-primary' : 'hover:bg-gray-100 dark:hover:bg-gray-700'"
        @click="toggleHeading(2)"
      >
        H2
      </button>
      <button
        class="px-2 py-1 rounded text-xs font-medium transition-colors"
        :class="isActive('heading', { level: 3 }) ? 'bg-linden-primary/20 text-linden-primary' : 'hover:bg-gray-100 dark:hover:bg-gray-700'"
        @click="toggleHeading(3)"
      >
        H3
      </button>

      <div class="w-px h-4 bg-gray-300 dark:bg-gray-600 mx-1" />

      <!-- 粗体/斜体 -->
      <button
        class="px-2 py-1 rounded text-xs font-bold transition-colors"
        :class="isActive('bold') ? 'bg-linden-primary/20 text-linden-primary' : 'hover:bg-gray-100 dark:hover:bg-gray-700'"
        @click="toggle('bold')"
      >
        B
      </button>
      <button
        class="px-2 py-1 rounded text-xs italic transition-colors"
        :class="isActive('italic') ? 'bg-linden-primary/20 text-linden-primary' : 'hover:bg-gray-100 dark:hover:bg-gray-700'"
        @click="toggle('italic')"
      >
        I
      </button>

      <div class="w-px h-4 bg-gray-300 dark:bg-gray-600 mx-1" />

      <!-- 列表 -->
      <button
        class="px-2 py-1 rounded text-xs transition-colors"
        :class="isActive('bulletList') ? 'bg-linden-primary/20 text-linden-primary' : 'hover:bg-gray-100 dark:hover:bg-gray-700'"
        @click="toggleList('bulletList')"
      >
        • 列表
      </button>
      <button
        class="px-2 py-1 rounded text-xs transition-colors"
        :class="isActive('orderedList') ? 'bg-linden-primary/20 text-linden-primary' : 'hover:bg-gray-100 dark:hover:bg-gray-700'"
        @click="toggleList('orderedList')"
      >
        1. 列表
      </button>

      <div class="w-px h-4 bg-gray-300 dark:bg-gray-600 mx-1" />

      <!-- 引用 -->
      <button
        class="px-2 py-1 rounded text-xs transition-colors"
        :class="isActive('blockquote') ? 'bg-linden-primary/20 text-linden-primary' : 'hover:bg-gray-100 dark:hover:bg-gray-700'"
        @click="toggleBlockquote"
      >
        引用
      </button>

      <div class="w-px h-4 bg-gray-300 dark:bg-gray-600 mx-1" />

      <!-- 分场线 -->
      <button
        class="px-2 py-1 rounded text-xs transition-colors hover:bg-gray-100 dark:hover:bg-gray-700"
        @click="insertSceneBreak"
        title="插入分场线 (* * *)"
      >
        分场
      </button>

      <div class="w-px h-4 bg-gray-300 dark:bg-gray-600 mx-1" />

      <!-- AI 功能 -->
      <button
        class="px-2 py-1 rounded text-xs transition-colors hover:bg-blue-100 dark:hover:bg-blue-900 text-blue-600 dark:text-blue-400"
        @click="handleAIContinue"
        title="AI 续写"
      >
        ✨ 续写
      </button>
      <button
        class="px-2 py-1 rounded text-xs transition-colors hover:bg-purple-100 dark:hover:bg-purple-900 text-purple-600 dark:text-purple-400"
        @click="handleAIRewrite"
        title="AI 改写"
      >
        🔄 改写
      </button>
      <button
        class="px-2 py-1 rounded text-xs transition-colors hover:bg-green-100 dark:hover:bg-green-900 text-green-600 dark:text-green-400"
        @click="handleAIExpand"
        title="AI 扩写"
      >
        📝 扩写
      </button>
      <button
        class="px-2 py-1 rounded text-xs transition-colors hover:bg-yellow-100 dark:hover:bg-yellow-900 text-yellow-600 dark:text-yellow-400"
        @click="handleAIPolish"
        title="AI 润色"
      >
        💎 润色
      </button>
    </div>
  </BubbleMenu>
</template>

<style scoped>
.bubble-toolbar {
  font-size: 13px;
}
</style>
