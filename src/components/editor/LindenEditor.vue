<script setup lang="ts">
import { ref, watch, onBeforeUnmount, onMounted, onUnmounted } from "vue";
import { EditorContent, useEditor } from "@tiptap/vue-3";
import StarterKit from "@tiptap/starter-kit";
import Placeholder from "@tiptap/extension-placeholder";
import Underline from "@tiptap/extension-underline";
import Highlight from "@tiptap/extension-highlight";
import TextAlign from "@tiptap/extension-text-align";
import Link from "@tiptap/extension-link";
import TaskList from "@tiptap/extension-task-list";
import TaskItem from "@tiptap/extension-task-item";
import { useChapterStore } from "../../stores/chapter";
import { useMessage } from "naive-ui";
import { useEditorUI } from "../../composables/useEditorUI";
import BlockMenu from "./BlockMenu.vue";
import ContextMenu from "./ContextMenu.vue";
import ChapterElementBar from "./ChapterElementBar.vue";
import AICompletionPanel from "../ai/AICompletionPanel.vue";
import AIGenerationDialog from "../ai/AIGenerationDialog.vue";
import { SceneBreak } from "./extensions/sceneBreak";
import { DraggableHandle } from "./extensions/DraggableHandle";
import { SlashCommand } from "./extensions/SlashCommand";
import "./extensions/slashMenu.css";

const chapterStore = useChapterStore();
const message = useMessage();

// 编辑器 UI 共享状态（专注模式 / 打字机模式 / AI 生成对话框）
const { focusMode, typewriterMode, showAIGenerationDialog } = useEditorUI();

// 右键菜单
const showContextMenu = ref(false);
const contextMenuPosition = ref<{ x: number; y: number } | null>(null);
const contextMenuSelectedText = ref("");

// 块级菜单
const showBlockMenu = ref(false);
const blockMenuPosition = ref<{ x: number; y: number } | null>(null);

// AI 补全
const showAIPanel = ref(false);
const aiContextText = ref("");
const aiCursorPosition = ref<{ from: number; to: number } | null>(null);
const aiMode = ref<"complete" | "continue" | "rewrite" | "expand" | "polish">("complete");

// TipTap 编辑器实例
const editor = useEditor({
  extensions: [
    StarterKit.configure({
      heading: { levels: [1, 2, 3] },
    }),
    Placeholder.configure({
      placeholder: "输入 / 打开命令菜单，或开始写作...",
    }),
    Underline,
    Highlight.configure({ multicolor: true }),
    TextAlign.configure({
      types: ["heading", "paragraph"],
    }),
    Link.configure({
      openOnClick: false,
    }),
    TaskList,
    TaskItem.configure({
      nested: true,
    }),
    SceneBreak,
    DraggableHandle,
    SlashCommand,
  ],
  content: "",
  onUpdate: ({ editor }) => {
    const json = editor.getJSON();
    const text = editor.getText();
    chapterStore.updateContent(JSON.stringify(json), text);

    if (typewriterMode.value) {
      scrollCaretToCenter();
    }
  },
  onSelectionUpdate: ({ editor }) => {
    const { from, to } = editor.state.selection;
    if (from !== to) {
      const text = editor.state.doc.textBetween(from, to, "\n");
      contextMenuSelectedText.value = text;
    } else {
      contextMenuSelectedText.value = "";
    }
  },
});

// 监听编辑器事件 — 用 watch + immediate 确保编辑器实例就绪后再注册
watch(
  editor,
  (ed, _, onCleanup) => {
    if (!ed) return;

    const handleBlockClick = ({ pos }: { pos: number }) => {
      if (!editor.value) return;
      const coords = editor.value.view.coordsAtPos(pos);
      const rect = editor.value.view.dom.getBoundingClientRect();

      blockMenuPosition.value = {
        x: rect.left - 60,
        y: coords.top - 10,
      };
      showBlockMenu.value = true;
    };

    const handleAIAction = (action: string) => {
      openAICompletion(action as any);
    };

    ed.on("blockHandleClick" as any, handleBlockClick);
    ed.on("aiAction" as any, handleAIAction);

    onCleanup(() => {
      ed.off("blockHandleClick" as any, handleBlockClick);
      ed.off("aiAction" as any, handleAIAction);
    });
  },
  { immediate: true }
);

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

    try {
      const json = JSON.parse(content.content_json);
      editor.value.commands.setContent(json);
    } catch {
      try {
        editor.value.commands.setContent(content.content_text || "");
      } catch {
        // 编辑器视图可能尚未就绪，忽略
      }
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

    if (saveTimer) clearTimeout(saveTimer);

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

function handleAIAccept(content: string) {
  if (!editor.value) return;

  const { from, to } = editor.value.state.selection;

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
  
  editor.value.chain().focus().insertContent(content).run();
  message.success("AI 生成内容已插入");
}

// 右键菜单处理
function handleContextMenu(e: MouseEvent) {
  if (!editor.value) return;
  
  e.preventDefault();
  
  // 获取当前选区
  const { from, to } = editor.value.state.selection;
  if (from !== to) {
    contextMenuSelectedText.value = editor.value.state.doc.textBetween(from, to, "\n");
  } else {
    contextMenuSelectedText.value = "";
  }
  
  contextMenuPosition.value = {
    x: e.clientX,
    y: e.clientY,
  };
  showContextMenu.value = true;
}

// 快捷键监听
function handleKeydown(e: KeyboardEvent) {
  if ((e.ctrlKey || e.metaKey) && e.key === "k") {
    e.preventDefault();
    openAICompletion();
  }
  if ((e.ctrlKey || e.metaKey) && e.shiftKey && (e.key === "g" || e.key === "G")) {
    e.preventDefault();
    openAIGeneration();
  }
}

// 点击编辑器时关闭右键菜单
function handleEditorClick() {
  if (showContextMenu.value) {
    showContextMenu.value = false;
  }
}

// 存储编辑器 DOM 引用，避免 onUnmounted 时访问已销毁的 view
let editorDom: HTMLElement | null = null;

onMounted(() => {
  window.addEventListener("keydown", handleKeydown);

  // 绑定右键菜单 — view 可能尚未就绪，用 try-catch 保护
  try {
    editorDom = editor.value?.view.dom ?? null;
    if (editorDom) {
      editorDom.addEventListener("contextmenu", handleContextMenu);
    }
  } catch {
    // 编辑器视图尚未挂载，跳过
  }
});

onUnmounted(() => {
  window.removeEventListener("keydown", handleKeydown);

  // 使用存储的 DOM 引用，避免访问已销毁的 editor.view
  if (editorDom) {
    editorDom.removeEventListener("contextmenu", handleContextMenu);
    editorDom = null;
  }
});
</script>

<template>
  <div class="linden-editor flex flex-col h-full relative" :class="{ 'focus-mode': focusMode }">
    <!-- 章节元素关联栏 -->
    <ChapterElementBar v-show="!focusMode" />

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

    <!-- 块级菜单（点击拖拽手柄弹出） -->
    <BlockMenu
      v-if="showBlockMenu && editor"
      :editor="editor"
      :position="blockMenuPosition"
      @close="showBlockMenu = false"
    />

    <!-- 右键上下文菜单 -->
    <ContextMenu
      v-if="editor"
      :editor="editor"
      :visible="showContextMenu"
      :position="contextMenuPosition"
      :selected-text="contextMenuSelectedText"
      @update:visible="showContextMenu = $event"
    />

    <!-- 编辑器内容区（纸张风格） -->
    <div
      class="editor-scroll-area flex-1 overflow-auto relative"
      :class="{ 'typewriter-scroll': typewriterMode }"
      @click="handleEditorClick"
    >
      <div class="editor-paper mx-auto max-w-3xl px-12 py-10">
        <EditorContent :editor="editor" class="max-w-none" />
      </div>
    </div>
  </div>
</template>

<style scoped>
/* 编辑器滚动区（"桌面"背景） */
.editor-scroll-area {
  background-color: #f3f4f6;
  padding: 1rem 1.5rem;
  overflow: auto;
  position: relative;
}

/* 纸张容器 */
.editor-paper {
  position: relative;
  background-color: #ffffff;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.06), 0 1px 2px rgba(0, 0, 0, 0.04);
  border-radius: 8px;
  min-height: calc(100% - 2rem);
}

/* 打字机模式：内容区上下留白，使光标可居中 */
.typewriter-scroll {
  padding-top: 40vh;
  padding-bottom: 40vh;
}

/* 拖拽手柄容器样式 — 外部浮动元素，绝对定位到 .editor-paper */
:deep(.block-handle-container) {
  position: absolute;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  opacity: 0;
  transition: opacity 0.12s ease;
  z-index: 10;
  pointer-events: none;
}

:deep(.block-handle-container.visible) {
  opacity: 1;
  pointer-events: auto;
}

:deep(.block-handle-drag) {
  width: 20px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  color: #9ca3af;
  cursor: grab;
  transition: all 0.12s ease;
  background: transparent;
  opacity: 0.7;
}

:deep(.block-handle-drag:hover) {
  background-color: #f3f4f6;
  color: #374151;
  cursor: grab;
  opacity: 1;
}

:deep(.block-handle-drag:active) {
  cursor: grabbing;
}

/* TipTap 编辑器基础样式 */
:deep(.tiptap) {
  outline: none;
  min-height: 100%;
  color: #374151;

  p {
    margin-bottom: 0.75em;
    line-height: 1.7;
  }

  h1, h2, h3 {
    margin-top: 1.5em;
    margin-bottom: 0.5em;
    font-weight: 600;
    color: #111827;
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

  a {
    color: #3b82f6;
    text-decoration: underline;
    cursor: pointer;
  }

  /* Task list styles */
  ul[data-type="taskList"] {
    list-style: none;
    padding-left: 0;

    li {
      display: flex;
      align-items: flex-start;
      gap: 0.5em;

      > label {
        flex-shrink: 0;
      }

      > div {
        flex: 1;
      }

      &[data-checked="true"] > div {
        text-decoration: line-through;
        opacity: 0.5;
      }
    }
  }

  /* Highlight styles */
  mark {
    background-color: #fef08a;
    border-radius: 2px;
    padding: 0 2px;
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
</style>

<!--
  暗色模式样式 — 必须用非 scoped 的 <style> 块。
  原因：.dark 类挂在 <html> 上，超出组件作用域。
  scoped 会编译成 .dark[data-v-xxx] .tiptap，但 <html> 没有 data-v-xxx
  属性，导致选择器永远不匹配。用 .linden-editor 限定作用范围避免泄漏。
-->
<style>
.dark .linden-editor .tiptap {
  color: #f3f4f6;
}

.dark .linden-editor .tiptap h1,
.dark .linden-editor .tiptap h2,
.dark .linden-editor .tiptap h3 {
  color: #ffffff;
}

.dark .linden-editor .tiptap blockquote {
  border-left-color: #4b5563;
  color: #9ca3af;
}

.dark .linden-editor .tiptap code {
  background-color: #374151;
}

.dark .linden-editor .tiptap pre {
  background-color: #111827;
}

.dark .linden-editor .tiptap mark {
  background-color: #854d0e;
  color: #fef3c7;
}

.dark .linden-editor .tiptap a {
  color: #60a5fa;
}

/* 暗色模式：编辑器滚动区 + 纸张 */
.dark .linden-editor .editor-scroll-area {
  background-color: #0f172a;
}

.dark .linden-editor .editor-paper {
  background-color: #1e293b;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.3), 0 1px 2px rgba(0, 0, 0, 0.2);
}

/* 暗色模式拖拽手柄 */
.dark .block-handle-add:hover,
.dark .block-handle-drag:hover {
  background-color: #374151;
  color: #d1d5db;
}
</style>
