<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import type { Editor } from "@tiptap/vue-3";
import { Selection } from "@tiptap/pm/state";

const props = defineProps<{
  editor: Editor;
  position: {
    x: number;
    y: number;
  } | null;
}>();

const emit = defineEmits<{
  (e: "close"): void;
}>();

const menuRef = ref<HTMLElement | null>(null);

function isActive(name: string, attrs?: Record<string, any>) {
  return props.editor.isActive(name, attrs);
}

function toggleHeading(level: 1 | 2 | 3) {
  props.editor.chain().focus().toggleHeading({ level }).run();
  emit("close");
}

function toggleParagraph() {
  props.editor.chain().focus().setParagraph().run();
  emit("close");
}

function toggleList(type: "bulletList" | "orderedList") {
  props.editor.chain().focus().toggleList(type, "listItem").run();
  emit("close");
}

function toggleTodo() {
  props.editor.chain().focus().toggleBulletList().toggleTaskList().run();
  emit("close");
}

function toggleBlockquote() {
  props.editor.chain().focus().toggleBlockquote().run();
  emit("close");
}

function toggleCodeBlock() {
  props.editor.chain().focus().toggleCodeBlock().run();
  emit("close");
}

function insertSceneBreak() {
  props.editor.chain().focus().setSceneBreak().run();
  emit("close");
}

// 获取当前光标所在的可操作块级节点及其父上下文
// 规则：
// - 若光标在列表内（<ul>/<ol> 下的 <li>），操作单元为 <li>（其父是列表），在列表内移动
// - 若光标在块引用/代码块内的段落，操作单元为段落（父是容器节点）
// - 否则为顶级块（段落/标题等），父是 doc
function getCurrentBlockContext() {
  const { $from } = props.editor.state.selection;
  // depth 0 = doc, 向上找到第一个 isBlock 祖先作为移动单元
  // 典型: doc(depth=0) -> p(depth=1, inline 文本在 depth=2)
  // doc -> ul(depth=1) -> li(depth=2) -> p(depth=3, inline depth=4)
  // 目标 depth: 最靠近文本层且 isBlock=true 的节点
  for (let d = $from.depth; d >= 1; d--) {
    const node = $from.node(d);
    if (!node) continue;
    const parent = $from.node(d - 1);
    // 当父节点允许多个 block 子节点（或至少 2 个子节点）时，此层为移动层
    // 对 list item 特殊处理: 当父是 ul/ol 时，li 就是移动单元
    if (node.isBlock && parent && parent.childCount >= 1) {
      return {
        node,
        pos: $from.before(d),   // 此节点在文档中的位置
        parent,
        parentPos: d - 1 === 0 ? 0 : $from.before(d - 1), // 父节点位置
        index: $from.index(d - 1), // 在父中的索引
        depth: d,
      };
    }
  }
  // 兜底：顶级块
  if ($from.depth >= 1) {
    return {
      node: $from.node(1),
      pos: $from.before(1),
      parent: $from.node(0),
      parentPos: 0,
      index: $from.index(0),
      depth: 1,
    };
  }
  return null;
}

function duplicateBlock() {
  const ctx = getCurrentBlockContext();
  if (!ctx) {
    emit("close");
    return;
  }
  const { pos, node } = ctx;
  const insertPos = pos + node.nodeSize;
  props.editor
    .chain()
    .focus()
    .insertContentAt(insertPos, [node.toJSON()])
    .run();
  emit("close");
}

function deleteBlock() {
  const ctx = getCurrentBlockContext();
  if (!ctx) {
    emit("close");
    return;
  }
  const { pos, node } = ctx;
  props.editor
    .chain()
    .focus()
    .deleteRange({ from: pos, to: pos + node.nodeSize })
    .run();
  emit("close");
}

// 通用的同层块移动函数
function swapWithSibling(direction: -1 | 1) {
  const ctx = getCurrentBlockContext();
  if (!ctx) return false;
  const { pos, node, parent, index } = ctx;
  const targetIndex = index + direction;
  if (targetIndex < 0 || targetIndex >= parent.childCount) return false;
  const sibling = parent.child(targetIndex);

  let from: number, to: number, newContent: typeof node[];
  if (direction === -1) {
    // 上移：[... sibling, node ...] -> [... node, sibling ...]
    from = pos - sibling.nodeSize;
    to = pos + node.nodeSize;
    newContent = [node, sibling];
  } else {
    // 下移：[... node, sibling ...] -> [... sibling, node ...]
    from = pos;
    to = pos + node.nodeSize + sibling.nodeSize;
    newContent = [sibling, node];
  }

  const { state } = props.editor;
  const tr = state.tr.replaceWith(from, to, newContent);

  // 关键：把选区映射后放入"当前块"内，保证后续操作还能定位到同一个逻辑块
  // 移动后：
  //   direction=-1 (上移): 当前块占据新内容 [from, from + node.nodeSize)
  //   direction=1  (下移): 当前块占据新内容 [from + sibling.nodeSize, from + sibling.nodeSize + node.nodeSize)
  const nodeStartAfter = direction === -1 ? from : from + sibling.nodeSize;
  try {
    // 节点内部位置（节点起始 + 1 即进入内容区）
    const insidePos = Math.min(
      nodeStartAfter + Math.max(1, Math.min(state.selection.from - pos, node.nodeSize - 2)),
      nodeStartAfter + node.nodeSize - 1,
    );
    if (insidePos >= 0 && insidePos <= tr.doc.content.size) {
      const $newPos = tr.doc.resolve(insidePos);
      tr.setSelection(Selection.near($newPos, 1));
    }
  } catch {
    /* 选区设置失败就让 PM 自动映射，通常也能工作 */
  }
  tr.scrollIntoView();

  try {
    const view = props.editor.view;
    if (view && !(view as any).isDestroyed) {
      view.dispatch(tr);
      return true;
    }
  } catch {
    /* Tiptap v3 中 editor.view 可能在卸载时抛异常 */
  }
  return false;
}

function moveBlockUp() {
  swapWithSibling(-1);
  emit("close");
}

function moveBlockDown() {
  swapWithSibling(1);
  emit("close");
}

function handleClickOutside(event: MouseEvent) {
  if (menuRef.value && !menuRef.value.contains(event.target as Node)) {
    emit("close");
  }
}

function handleKeydown(event: KeyboardEvent) {
  if (event.key === "Escape") {
    emit("close");
  }
}

onMounted(() => {
  document.addEventListener("mousedown", handleClickOutside);
  document.addEventListener("keydown", handleKeydown);
});

onUnmounted(() => {
  document.removeEventListener("mousedown", handleClickOutside);
  document.removeEventListener("keydown", handleKeydown);
});
</script>

<template>
  <Teleport to="body">
    <div
      v-if="position"
      ref="menuRef"
      class="block-menu fixed z-50 bg-white dark:bg-gray-800 rounded-lg shadow-xl border border-gray-200 dark:border-gray-700 py-1.5 min-w-[200px]"
      :style="{ top: position.y + 'px', left: position.x + 'px' }"
    >
      <!-- 块类型转换 -->
      <div class="menu-section">
        <div class="ctx-group">转换为</div>
        <button
          class="ctx-item"
          :class="{ 'is-active': isActive('heading', { level: 1 }) }"
          @click="toggleHeading(1)"
        >
          <span class="ctx-icon"><span class="i-carbon-heading text-sm"></span></span>
          <div class="ctx-content">
            <div class="ctx-title">一级标题</div>
            <div class="ctx-desc">大标题</div>
          </div>
        </button>
        <button
          class="ctx-item"
          :class="{ 'is-active': isActive('heading', { level: 2 }) }"
          @click="toggleHeading(2)"
        >
          <span class="ctx-icon"><span class="i-carbon-heading text-sm"></span></span>
          <div class="ctx-content">
            <div class="ctx-title">二级标题</div>
            <div class="ctx-desc">中标题</div>
          </div>
        </button>
        <button
          class="ctx-item"
          :class="{ 'is-active': isActive('heading', { level: 3 }) }"
          @click="toggleHeading(3)"
        >
          <span class="ctx-icon"><span class="i-carbon-heading text-sm"></span></span>
          <div class="ctx-content">
            <div class="ctx-title">三级标题</div>
            <div class="ctx-desc">小标题</div>
          </div>
        </button>
        <button
          class="ctx-item"
          :class="{ 'is-active': isActive('paragraph') }"
          @click="toggleParagraph"
        >
          <span class="ctx-icon"><span class="i-carbon-document text-sm"></span></span>
          <div class="ctx-content">
            <div class="ctx-title">正文</div>
            <div class="ctx-desc">普通段落</div>
          </div>
        </button>
      </div>

      <div class="ctx-divider" />

      <!-- 块样式 -->
      <div class="menu-section">
        <div class="ctx-group">块样式</div>
        <button
          class="ctx-item"
          :class="{ 'is-active': isActive('bulletList') }"
          @click="toggleList('bulletList')"
        >
          <span class="ctx-icon"><span class="i-carbon-list-bulleted text-sm"></span></span>
          <div class="ctx-content">
            <div class="ctx-title">无序列表</div>
            <div class="ctx-desc">项目符号</div>
          </div>
        </button>
        <button
          class="ctx-item"
          :class="{ 'is-active': isActive('orderedList') }"
          @click="toggleList('orderedList')"
        >
          <span class="ctx-icon"><span class="i-carbon-list-numbered text-sm"></span></span>
          <div class="ctx-content">
            <div class="ctx-title">有序列表</div>
            <div class="ctx-desc">编号列表</div>
          </div>
        </button>
        <button
          class="ctx-item"
          :class="{ 'is-active': isActive('taskList') }"
          @click="toggleTodo"
        >
          <span class="ctx-icon"><span class="i-carbon-checkbox text-sm"></span></span>
          <div class="ctx-content">
            <div class="ctx-title">待办事项</div>
            <div class="ctx-desc">复选框列表</div>
          </div>
        </button>
        <button
          class="ctx-item"
          :class="{ 'is-active': isActive('blockquote') }"
          @click="toggleBlockquote"
        >
          <span class="ctx-icon"><span class="i-carbon-quotes text-sm"></span></span>
          <div class="ctx-content">
            <div class="ctx-title">引用</div>
            <div class="ctx-desc">引用块</div>
          </div>
        </button>
        <button
          class="ctx-item"
          :class="{ 'is-active': isActive('codeBlock') }"
          @click="toggleCodeBlock"
        >
          <span class="ctx-icon"><span class="i-carbon-code text-sm"></span></span>
          <div class="ctx-content">
            <div class="ctx-title">代码块</div>
            <div class="ctx-desc">代码片段</div>
          </div>
        </button>
        <button
          class="ctx-item"
          @click="insertSceneBreak"
        >
          <span class="ctx-icon"><span class="i-carbon-cut text-sm"></span></span>
          <div class="ctx-content">
            <div class="ctx-title">分场线</div>
            <div class="ctx-desc">场景分隔</div>
          </div>
        </button>
      </div>

      <div class="ctx-divider" />

      <!-- 块操作 -->
      <div class="menu-section">
        <div class="ctx-group">操作</div>
        <button class="ctx-item" @click="moveBlockUp">
          <span class="ctx-icon"><span class="i-carbon-arrow-up text-sm"></span></span>
          <div class="ctx-content">
            <div class="ctx-title">上移</div>
            <div class="ctx-desc">向上移动块</div>
          </div>
        </button>
        <button class="ctx-item" @click="moveBlockDown">
          <span class="ctx-icon"><span class="i-carbon-arrow-down text-sm"></span></span>
          <div class="ctx-content">
            <div class="ctx-title">下移</div>
            <div class="ctx-desc">向下移动块</div>
          </div>
        </button>
        <button class="ctx-item" @click="duplicateBlock">
          <span class="ctx-icon"><span class="i-carbon-copy text-sm"></span></span>
          <div class="ctx-content">
            <div class="ctx-title">复制</div>
            <div class="ctx-desc">复制当前块</div>
          </div>
        </button>
        <button class="ctx-item is-danger" @click="deleteBlock">
          <span class="ctx-icon"><span class="i-carbon-trash-can text-sm"></span></span>
          <div class="ctx-content">
            <div class="ctx-title">删除</div>
            <div class="ctx-desc">删除当前块</div>
          </div>
        </button>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.block-menu {
  max-height: 380px;
  overflow-y: auto;
  font-size: 13px;
}

.menu-section {
  padding: 2px 4px;
}

.ctx-group {
  padding: 3px 8px 1px;
  font-size: 10px;
  font-weight: 600;
  color: #6b7280;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.ctx-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 5px 8px;
  border: none;
  background: transparent;
  border-radius: 6px;
  color: #374151;
  transition: background-color 0.12s;
  cursor: pointer;
  text-align: left;
}

.ctx-item:hover,
.ctx-item.is-active {
  background-color: #f3f4f6;
}

.ctx-item.is-active {
  background-color: #e0e7ff;
  color: #4f46e5;
}

.ctx-item.is-danger {
  color: #dc2626;
}

.ctx-item.is-danger:hover {
  background-color: #fef2f2;
}

.ctx-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border-radius: 6px;
  background-color: #f9fafb;
  color: #6b7280;
  flex-shrink: 0;
}

.ctx-content {
  flex: 1;
  min-width: 0;
}

.ctx-title {
  font-size: 13px;
  font-weight: 500;
  color: #111827;
  line-height: 1.3;
}

.ctx-desc {
  font-size: 11px;
  color: #9ca3af;
  margin-top: 0;
}

.ctx-divider {
  height: 1px;
  background-color: #f3f4f6;
  margin: 3px 6px;
}

/* Dark mode */
.dark .ctx-item {
  color: #d1d5db;
}

.dark .ctx-item:hover {
  background-color: #374151;
}

.dark .ctx-item.is-active {
  background-color: rgba(99, 226, 183, 0.15);
  color: #e0e7ff;
}

.dark .ctx-item.is-danger {
  color: #f87171;
}

.dark .ctx-item.is-danger:hover {
  background-color: #450a0a;
}

.dark .ctx-icon {
  background-color: #374151;
  color: #9ca3af;
}

.dark .ctx-title {
  color: #f3f4f6;
}

.dark .ctx-desc {
  color: #6b7280;
}

.dark .ctx-group {
  color: #9ca3af;
}

.dark .ctx-divider {
  background-color: #374151;
}

/* Scrollbar */
.block-menu::-webkit-scrollbar {
  width: 6px;
}

.block-menu::-webkit-scrollbar-track {
  background: transparent;
}

.block-menu::-webkit-scrollbar-thumb {
  background-color: #d1d5db;
  border-radius: 3px;
}

.dark .block-menu::-webkit-scrollbar-thumb {
  background-color: #4b5563;
}
</style>
