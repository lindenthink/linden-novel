<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from "vue";
import type { Editor } from "@tiptap/vue-3";
import {
  sharedMenuItems,
  groupItemsByCategory,
  contextCategoryOrder,
  type MenuItem,
} from "./menuItems";

const props = defineProps<{
  editor: Editor;
  visible: boolean;
  position: { x: number; y: number } | null;
  selectedText: string;
}>();

const emit = defineEmits<{
  (e: "update:visible", value: boolean): void;
}>();

const menuRef = ref<HTMLElement | null>(null);
const activeSubmenu = ref<"none" | "textColor" | "highlight">("none");

const hasSelection = computed(() => {
  if (!props.editor) return false;
  const { from, to } = props.editor.state.selection;
  return from !== to;
});

const isLink = computed(() => props.editor?.isActive("link"));

// 共享菜单项分组（AI 助手 + 块格式）
const sharedGroups = computed(() =>
  groupItemsByCategory(sharedMenuItems, contextCategoryOrder)
);

function isActive(name: string, attrs?: Record<string, any>) {
  return props.editor?.isActive(name, attrs);
}

function closeMenu() {
  activeSubmenu.value = "none";
  emit("update:visible", false);
}

function runAction(fn: () => void) {
  fn();
  closeMenu();
}

function runSharedCommand(item: MenuItem) {
  runAction(() => item.command({ editor: props.editor }));
}

function toggle(name: string, attrs?: Record<string, any>) {
  runAction(() => props.editor?.chain().focus().toggleMark(name, attrs).run());
}

function createLink() {
  if (!props.editor) return;
  const url = window.prompt("输入链接地址");
  if (url) {
    props.editor.chain().focus().toggleLink({ href: url }).run();
  }
  closeMenu();
}

function removeLink() {
  runAction(() => props.editor?.chain().focus().unsetLink().run());
}

function setTextColor(color: string) {
  runAction(() => props.editor?.chain().focus().setMark("textStyle", { color }).run());
}

function setHighlight(color: string) {
  runAction(() => props.editor?.chain().focus().setHighlight({ color }).run());
}

function clearHighlight() {
  runAction(() => props.editor?.chain().focus().unsetHighlight().run());
}

function copySelection() {
  if (!props.editor) return;
  const { from, to } = props.editor.state.selection;
  const text = props.editor.state.doc.textBetween(from, to, "\n");
  navigator.clipboard.writeText(text).catch(() => {});
  closeMenu();
}

function toggleSubmenu(name: "textColor" | "highlight") {
  activeSubmenu.value = activeSubmenu.value === name ? "none" : name;
}

function handleMenuMouseDown(e: MouseEvent) {
  e.stopPropagation();
}

function handleClickOutside(event: MouseEvent) {
  if (menuRef.value && !menuRef.value.contains(event.target as Node)) {
    closeMenu();
  }
}

function handleKeydown(event: KeyboardEvent) {
  if (event.key === "Escape") closeMenu();
}

onMounted(() => {
  document.addEventListener("mousedown", handleClickOutside);
  document.addEventListener("keydown", handleKeydown);
});

onUnmounted(() => {
  document.removeEventListener("mousedown", handleClickOutside);
  document.removeEventListener("keydown", handleKeydown);
});

const textColors = [
  { name: "默认", value: "#374151" },
  { name: "红色", value: "#ef4444" },
  { name: "橙色", value: "#f97316" },
  { name: "黄色", value: "#eab308" },
  { name: "绿色", value: "#22c55e" },
  { name: "蓝色", value: "#3b82f6" },
  { name: "紫色", value: "#a855f7" },
  { name: "粉色", value: "#ec4899" },
];

const highlightColors = [
  { name: "黄色", value: "#fef08a" },
  { name: "绿色", value: "#bbf7d0" },
  { name: "蓝色", value: "#bfdbfe" },
  { name: "粉色", value: "#fbcfe8" },
  { name: "紫色", value: "#ddd6fe" },
  { name: "橙色", value: "#fed7aa" },
];
</script>

<template>
  <Teleport to="body">
    <div
      v-if="visible && position"
      ref="menuRef"
      class="ctx-menu fixed z-50 bg-white dark:bg-gray-800 rounded-lg shadow-xl border border-gray-200 dark:border-gray-700 py-2 min-w-[240px] max-w-[280px]"
      :style="{ top: position.y + 'px', left: position.x + 'px' }"
      @mousedown="handleMenuMouseDown"
    >
      <!-- ═══ 共享菜单项（AI 助手 + 块格式）与斜杠菜单一致 ═══ -->
      <template v-for="(group, groupIdx) in sharedGroups" :key="group.category">
        <div v-if="groupIdx > 0" class="ctx-divider" />
        <div class="ctx-group">{{ group.category }}</div>
        <button
          v-for="item in group.items"
          :key="item.id"
          class="ctx-item"
          @click="runSharedCommand(item)"
        >
          <span class="ctx-icon"><span :class="item.icon"></span></span>
          <div class="ctx-content">
            <div class="ctx-title">{{ item.title }}</div>
            <div class="ctx-desc">{{ item.description }}</div>
          </div>
        </button>
      </template>

      <!-- ═══ 文字格式（仅右键菜单，需选区） ═══ -->
      <template v-if="hasSelection">
        <div class="ctx-divider" />
        <div class="ctx-group">文字格式</div>

        <button
          class="ctx-item"
          :class="{ 'is-active': isActive('bold') }"
          @click="toggle('bold')"
        >
          <span class="ctx-icon"><span class="i-carbon-text-bold text-sm"></span></span>
          <div class="ctx-content">
            <div class="ctx-title">加粗</div>
            <div class="ctx-desc">Ctrl + B</div>
          </div>
        </button>
        <button
          class="ctx-item"
          :class="{ 'is-active': isActive('italic') }"
          @click="toggle('italic')"
        >
          <span class="ctx-icon"><span class="i-carbon-text-italic text-sm"></span></span>
          <div class="ctx-content">
            <div class="ctx-title">斜体</div>
            <div class="ctx-desc">Ctrl + I</div>
          </div>
        </button>
        <button
          class="ctx-item"
          :class="{ 'is-active': isActive('underline') }"
          @click="toggle('underline')"
        >
          <span class="ctx-icon"><span class="i-carbon-text-underline text-sm"></span></span>
          <div class="ctx-content">
            <div class="ctx-title">下划线</div>
            <div class="ctx-desc">Ctrl + U</div>
          </div>
        </button>

        <!-- 文字颜色 -->
        <button
          class="ctx-item"
          :class="{ 'is-active': activeSubmenu === 'textColor' }"
          @click="toggleSubmenu('textColor')"
        >
          <span class="ctx-icon"><span class="i-carbon-text-color text-sm"></span></span>
          <div class="ctx-content">
            <div class="ctx-title">文字颜色</div>
            <div class="ctx-desc">设置文字颜色</div>
          </div>
          <span class="ctx-arrow">›</span>
        </button>
        <div v-if="activeSubmenu === 'textColor'" class="ctx-submenu">
          <button
            v-for="color in textColors"
            :key="color.value"
            class="ctx-color-swatch"
            :style="{ backgroundColor: color.value }"
            :title="color.name"
            @click="setTextColor(color.value)"
          />
        </div>

        <!-- 高亮 -->
        <button
          class="ctx-item"
          :class="{ 'is-active': activeSubmenu === 'highlight' }"
          @click="toggleSubmenu('highlight')"
        >
          <span class="ctx-icon"><span class="i-carbon-color-palette text-sm"></span></span>
          <div class="ctx-content">
            <div class="ctx-title">高亮</div>
            <div class="ctx-desc">标记重要内容</div>
          </div>
          <span class="ctx-arrow">›</span>
        </button>
        <div v-if="activeSubmenu === 'highlight'" class="ctx-submenu">
          <button
            v-for="color in highlightColors"
            :key="color.value"
            class="ctx-color-swatch"
            :style="{ backgroundColor: color.value }"
            :title="color.name"
            @click="setHighlight(color.value)"
          />
          <button class="ctx-clear-highlight" @click="clearHighlight">清除高亮</button>
        </div>

        <!-- 链接 -->
        <button v-if="!isLink" class="ctx-item" @click="createLink">
          <span class="ctx-icon"><span class="i-carbon-link text-sm"></span></span>
          <div class="ctx-content">
            <div class="ctx-title">添加链接</div>
            <div class="ctx-desc">Ctrl + K</div>
          </div>
        </button>
        <button v-else class="ctx-item" @click="removeLink">
          <span class="ctx-icon"><span class="i-carbon-link text-sm"></span></span>
          <div class="ctx-content">
            <div class="ctx-title">移除链接</div>
            <div class="ctx-desc">取消超链接</div>
          </div>
        </button>
      </template>

      <!-- ═══ 操作（仅右键菜单，需选区） ═══ -->
      <template v-if="hasSelection">
        <div class="ctx-divider" />
        <div class="ctx-group">操作</div>
        <button class="ctx-item" @click="copySelection">
          <span class="ctx-icon"><span class="i-carbon-copy text-sm"></span></span>
          <div class="ctx-content">
            <div class="ctx-title">复制</div>
            <div class="ctx-desc">Ctrl + C</div>
          </div>
        </button>
      </template>
    </div>
  </Teleport>
</template>

<style scoped>
.ctx-menu {
  font-size: 13px;
  max-height: 380px;
  overflow-y: auto;
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

.ctx-arrow {
  font-size: 16px;
  color: #9ca3af;
}

.ctx-divider {
  height: 1px;
  background-color: #f3f4f6;
  margin: 3px 6px;
}

.ctx-submenu {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  padding: 5px 16px 8px;
}

.ctx-color-swatch {
  width: 22px;
  height: 22px;
  border-radius: 5px;
  border: 2px solid transparent;
  cursor: pointer;
  transition: transform 0.12s, border-color 0.12s;
}

.ctx-color-swatch:hover {
  transform: scale(1.12);
  border-color: #d1d5db;
}

.ctx-clear-highlight {
  width: 100%;
  margin-top: 4px;
  padding: 4px;
  font-size: 11px;
  color: #6b7280;
  background: transparent;
  border: 1px solid #e5e7eb;
  border-radius: 5px;
  cursor: pointer;
}

.ctx-clear-highlight:hover {
  background-color: #f3f4f6;
}

/* Dark mode */
.dark .ctx-item {
  color: #d1d5db;
}

.dark .ctx-item:hover,
.dark .ctx-item.is-active {
  background-color: #374151;
}

.dark .ctx-item.is-active {
  background-color: #3730a3;
}

.dark .ctx-icon {
  background-color: #4b5563;
  color: #9ca3af;
}

.dark .ctx-title {
  color: #f3f4f6;
}

.dark .ctx-desc {
  color: #9ca3af;
}

.dark .ctx-group {
  color: #9ca3af;
}

.dark .ctx-arrow {
  color: #6b7280;
}

.dark .ctx-divider {
  background-color: #374151;
}

.dark .ctx-color-swatch:hover {
  border-color: #4b5563;
}

.dark .ctx-clear-highlight {
  color: #9ca3af;
  border-color: #4b5563;
}

.dark .ctx-clear-highlight:hover {
  background-color: #374151;
}

/* Scrollbar */
.ctx-menu::-webkit-scrollbar {
  width: 6px;
}

.ctx-menu::-webkit-scrollbar-track {
  background: transparent;
}

.ctx-menu::-webkit-scrollbar-thumb {
  background-color: #d1d5db;
  border-radius: 3px;
}

.dark .ctx-menu::-webkit-scrollbar-thumb {
  background-color: #4b5563;
}
</style>
