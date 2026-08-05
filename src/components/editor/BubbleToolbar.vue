<script setup lang="ts">
import { BubbleMenu } from "@tiptap/extension-bubble-menu";
import type { Editor } from "@tiptap/vue-3";

const props = defineProps<{
  editor: Editor;
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
</script>

<template>
  <BubbleMenu :editor="editor" :tippy-options="{ duration: 100 }">
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
    </div>
  </BubbleMenu>
</template>

<style scoped>
.bubble-toolbar {
  font-size: 13px;
}
</style>
