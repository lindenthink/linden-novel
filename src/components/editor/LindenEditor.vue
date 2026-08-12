<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from "vue";
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
import { useEditorSettings } from "../../composables/useEditorSettings";
import { listChapterElements } from "../../api/element";
import BlockMenu from "./BlockMenu.vue";
import ChapterElementBar from "./ChapterElementBar.vue";
import AICompletionPanel from "../ai/AICompletionPanel.vue";
import AIGenerationDialog from "../ai/AIGenerationDialog.vue";
import { SceneBreak } from "./extensions/sceneBreak";
import { DraggableHandle } from "./extensions/DraggableHandle";
import { SlashCommand } from "./extensions/SlashCommand";
import "./extensions/slashMenu.css";

const chapterStore = useChapterStore();
const message = useMessage();
const { isAutoSaveEnabled } = useEditorSettings();

// 抑制标志：通过 setContent 同步内容时，onUpdate 不应标记为 dirty
let suppressOnUpdate = false;

// 编辑器 UI 共享状态（AI 生成对话框）
const { showAIGenerationDialog } = useEditorUI();

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
      placeholder:
        "输入 / 打开命令菜单，或开始写作...\n\n快捷键：Ctrl+B 加粗 · Ctrl+I 斜体 · Ctrl+U 下划线 · Ctrl+Shift+8 无序列表 · Ctrl+Shift+7 有序列表 · Ctrl+Shift+- 分场线 · Ctrl+Z 撤销 · Ctrl+Shift+Z 重做",
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
    if (suppressOnUpdate) return;
    const json = editor.getJSON();
    const text = editor.getText();
    chapterStore.updateContent(JSON.stringify(json), text);
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

// 监听 activeContent 变化，同步到编辑器
watch(
  () => chapterStore.activeContent,
  (content) => {
    if (!editor.value || !content) return;

    suppressOnUpdate = true;
    try {
      const json = JSON.parse(content.content_json);
      editor.value.commands.setContent(json);
    } catch {
      try {
        editor.value.commands.setContent(content.content_text || "");
      } catch {
        // 编辑器视图可能尚未就绪，忽略
      }
    } finally {
      suppressOnUpdate = false;
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
    if (!isAutoSaveEnabled.value) return;

    if (saveTimer) clearTimeout(saveTimer);

    saveTimer = setTimeout(async () => {
      await chapterStore.flushSave();
    }, 1500);
  }
);

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
async function openAIGeneration() {
  if (!editor.value) return;
  
  const activeChapterId = chapterStore.activeChapterId;
  if (!activeChapterId) {
    message.warning("请先选择一个章节");
    return;
  }
  
  // 检查章节是否关联了角色，未关联则仅提示不打开对话框
  try {
    const elements = await listChapterElements(activeChapterId);
    const hasCharacter = elements.some(el => el.element_type === 'character');
    if (!hasCharacter) {
      message.warning("当前章节未关联角色，请先在侧边栏关联关键角色、故事线等元素后再使用 AI 生成。");
      return;
    }
  } catch (e) {
    console.error("加载章节元素失败:", e);
  }
  
  showAIGenerationDialog.value = true;
}

function handleAIGenerationApply(content: string) {
  if (!editor.value) return;
  
  editor.value.chain().focus().insertContent(content).run();
  message.success("AI 生成内容已插入");
}

// 快捷键监听
function handleKeydown(e: KeyboardEvent) {
  if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "s") {
    e.preventDefault();
    if (!chapterStore.dirty) {
      message.info("没有需要保存的更改");
      return;
    }
    if (saveTimer) {
      clearTimeout(saveTimer);
      saveTimer = null;
    }
    chapterStore.flushSave().then((wc) => {
      if (wc !== null) {
        message.success("已保存");
      }
    });
  }
  if ((e.ctrlKey || e.metaKey) && e.key === "k") {
    e.preventDefault();
    openAICompletion();
  }
  if ((e.ctrlKey || e.metaKey) && e.shiftKey && (e.key === "g" || e.key === "G")) {
    e.preventDefault();
    openAIGeneration();
  }
}

onMounted(() => {
  window.addEventListener("keydown", handleKeydown);
});

onUnmounted(() => {
  window.removeEventListener("keydown", handleKeydown);

  if (saveTimer) clearTimeout(saveTimer);
});
</script>

<template>
  <div class="linden-editor flex flex-col h-full relative">
    <!-- 章节元素关联栏 -->
    <ChapterElementBar />

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

    <!-- 编辑器内容区（纸张风格） -->
    <div
      class="editor-scroll-area flex-1 overflow-auto relative"
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
  cursor: pointer;
  transition: all 0.12s ease;
  background: transparent;
  opacity: 0.7;
}

:deep(.block-handle-drag:hover) {
  background-color: #f3f4f6;
  color: #374151;
  cursor: pointer;
  opacity: 1;
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
    white-space: pre-wrap;
    font-size: 13px;
    line-height: 1.8;
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
