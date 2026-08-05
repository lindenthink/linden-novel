<script setup lang="ts">
import { ref, watch, onBeforeUnmount, onMounted, onUnmounted } from "vue";
import { EditorContent, useEditor } from "@tiptap/vue-3";
import StarterKit from "@tiptap/starter-kit";
import Placeholder from "@tiptap/extension-placeholder";
import { useChapterStore } from "../../stores/chapter";
import { useMessage } from "naive-ui";
import BubbleToolbar from "./BubbleToolbar.vue";
import ChapterElementBar from "./ChapterElementBar.vue";
import AICompletionPanel from "../ai/AICompletionPanel.vue";
import AIGenerationDialog from "../ai/AIGenerationDialog.vue";
import { SceneBreak } from "./extensions/sceneBreak";

const chapterStore = useChapterStore();
const message = useMessage();

// 专注模式 / 打字机模式
const focusMode = ref(false);
const typewriterMode = ref(false);

// AI 补全
const showAIPanel = ref(false);
const aiContextText = ref("");
const aiCursorPosition = ref<{ from: number; to: number } | null>(null);
const aiMode = ref<"complete" | "continue" | "rewrite" | "expand" | "polish">("complete");

// AI 生成
const showAIGenerationDialog = ref(false);

// TipTap 编辑器实例
const editor = useEditor({
  extensions: [
    StarterKit.configure({
      heading: { levels: [1, 2, 3] },
    }),
    Placeholder.configure({
      placeholder: "开始写作...",
    }),
    SceneBreak,
  ],
  content: "",
  onUpdate: ({ editor }) => {
    // 内容变化时更新 store
    const json = editor.getJSON();
    const text = editor.getText();
    chapterStore.updateContent(JSON.stringify(json), text);

    // 打字机模式：保持光标在视口中央
    if (typewriterMode.value) {
      scrollCaretToCenter();
    }
  },
});

// 打字机模式：滚动使光标位于视口中央
function scrollCaretToCenter() {
  if (!editor.value) return;
  const { state, view } = editor.value;
  const { from } = state.selection;
  const coords = view.coordsAtPos(from);
  const scrollContainer = view.dom.parentElement;
  if (scrollContainer) {
    const containerHeight = scrollContainer.clientHeight;
    scrollContainer.scrollTop = coords.top - scrollContainer.offsetTop - containerHeight / 2;
  }
}

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

// 自动保存（debounce 1.5 秒）
let saveTimer: ReturnType<typeof setTimeout> | null = null;
watch(
  () => chapterStore.dirty,
  (isDirty) => {
    if (!isDirty) return;

    // 清除之前的定时器
    if (saveTimer) clearTimeout(saveTimer);

    // 1.5 秒后自动保存
    saveTimer = setTimeout(async () => {
      await chapterStore.flushSave();
    }, 1500);
  }
);

onBeforeUnmount(() => {
  if (saveTimer) clearTimeout(saveTimer);
});

// AI 补全功能
function openAICompletion(mode: "complete" | "continue" | "rewrite" | "expand" | "polish" = "complete") {
  if (!editor.value) return;

  const { from, to } = editor.value.state.selection;
  const text = editor.value.state.doc.textBetween(from, to, "\n");

  // 如果没有选中文本，获取当前段落作为上下文
  if (!text) {
    const paragraph = editor.value.state.doc.resolve(from).parent;
    const paragraphText = paragraph.textContent;
    if (paragraphText) {
      aiContextText.value = paragraphText;
    } else {
      message.warning("请先选择文本或将光标放在段落中");
      return;
    }
  } else {
    aiContextText.value = text;
  }

  aiCursorPosition.value = { from, to };
  aiMode.value = mode;
  showAIPanel.value = true;
}

// 气泡菜单 AI 功能
function handleAIContinue() {
  openAICompletion("continue");
}

function handleAIRewrite() {
  openAICompletion("rewrite");
}

function handleAIExpand() {
  openAICompletion("expand");
}

function handleAIPolish() {
  openAICompletion("polish");
}

function handleAIAccept(content: string) {
  if (!editor.value) return;

  const { from, to } = editor.value.state.selection;

  // 如果有选中文本，替换；否则在光标处插入
  if (from !== to) {
    editor.value.chain().focus().insertContent(content).run();
  } else {
    editor.value.chain().focus().insertContentAt(to, content).run();
  }

  showAIPanel.value = false;
  message.success("AI 补全已应用");
}

function handleAIReject() {
  showAIPanel.value = false;
  message.info("已拒绝 AI 补全");
}

function handleAIClose() {
  showAIPanel.value = false;
}

// AI 生成功能
function openAIGeneration() {
  if (!editor.value) return;
  
  const activeChapterId = chapterStore.activeChapterId;
  if (!activeChapterId) {
    message.warning("请先选择一个章节");
    return;
  }
  
  showAIGenerationDialog.value = true;
}

function handleAIGenerationApply(content: string) {
  if (!editor.value) return;
  
  // 在光标位置插入生成的内容
  editor.value.chain().focus().insertContent(content).run();
  message.success("AI 生成内容已插入");
}

// 快捷键监听
function handleKeydown(e: KeyboardEvent) {
  // Ctrl+K 或 Cmd+K 打开 AI 补全
  if ((e.ctrlKey || e.metaKey) && e.key === "k") {
    e.preventDefault();
    openAICompletion();
  }
}

onMounted(() => {
  window.addEventListener("keydown", handleKeydown);
});

onUnmounted(() => {
  window.removeEventListener("keydown", handleKeydown);
});
</script>

<template>
  <div class="linden-editor flex flex-col h-full" :class="{ 'focus-mode': focusMode }">
    <!-- 模式切换栏 -->
    <div class="flex items-center justify-between gap-2 px-4 py-1 border-b border-gray-100 dark:border-gray-800 flex-shrink-0">
      <div class="flex items-center gap-2">
        <button
          class="text-xs px-2 py-0.5 rounded transition-colors bg-blue-50 dark:bg-blue-900/20 text-blue-600 dark:text-blue-400 hover:bg-blue-100 dark:hover:bg-blue-900/30"
          @click="openAICompletion('complete')"
          title="AI 助手 (Ctrl+K)"
        >
          ✨ AI 助手
        </button>
      </div>
      <div class="flex items-center gap-2">
        <button
          class="text-xs px-2 py-0.5 rounded transition-colors bg-purple-50 dark:bg-purple-900/20 text-purple-600 dark:text-purple-400 hover:bg-purple-100 dark:hover:bg-purple-900/30"
          @click="openAIGeneration"
          title="AI 生成"
        >
          🤖 AI 生成
        </button>
        <button
          class="text-xs px-2 py-0.5 rounded transition-colors"
          :class="focusMode ? 'bg-linden-primary/20 text-linden-primary' : 'text-gray-400 hover:text-gray-600 dark:hover:text-gray-300'"
          @click="focusMode = !focusMode"
          title="专注模式：隐藏侧栏，沉浸写作"
        >
          专注
        </button>
        <button
          class="text-xs px-2 py-0.5 rounded transition-colors"
          :class="typewriterMode ? 'bg-linden-primary/20 text-linden-primary' : 'text-gray-400 hover:text-gray-600 dark:hover:text-gray-300'"
          @click="typewriterMode = !typewriterMode"
          title="打字机模式：光标始终居中"
        >
          打字机
        </button>
      </div>
    </div>

    <!-- 章节元素关联栏 -->
    <ChapterElementBar />

    <!-- 气泡工具栏 -->
    <BubbleToolbar
      v-if="editor"
      :editor="editor"
      @ai-continue="handleAIContinue"
      @ai-rewrite="handleAIRewrite"
      @ai-expand="handleAIExpand"
      @ai-polish="handleAIPolish"
    />

    <!-- AI 补全面板 -->
    <AICompletionPanel
      v-model:visible="showAIPanel"
      :context-text="aiContextText"
      :cursor-position="aiCursorPosition"
      :mode="aiMode"
      @accept="handleAIAccept"
      @reject="handleAIReject"
      @close="handleAIClose"
    />

    <!-- AI 生成对话框 -->
    <AIGenerationDialog
      v-model:show="showAIGenerationDialog"
      @apply="handleAIGenerationApply"
    />

    <!-- 编辑器内容区 -->
    <div class="flex-1 overflow-auto" :class="{ 'typewriter-scroll': typewriterMode }">
      <EditorContent :editor="editor" class="prose prose-sm max-w-none px-8 py-6" />
    </div>
  </div>
</template>

<style scoped>
/* 专注模式：隐藏模式切换栏和气泡工具栏 */
.focus-mode :deep(.linden-editor > div:first-child),
.focus-mode :deep(.bubble-menu) {
  display: none;
}

/* 打字机模式：内容区上下留白，使光标可居中 */
.typewriter-scroll {
  padding-top: 40vh;
  padding-bottom: 40vh;
}

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
