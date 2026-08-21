<script setup lang="ts">
import { ref, onMounted } from "vue";
import { NTooltip } from "naive-ui";
import * as settingsApi from "../../api/settings";

const props = defineProps<{
  /** 左栏初始宽度 (px)，默认 240 */
  defaultLeft?: number;
  /** 右栏初始宽度 (px)，默认 300 */
  defaultRight?: number;
}>();

const MIN_WIDTH = 180;
const MAX_LEFT = 400;
const MAX_RIGHT = 500;

// ---- 栏宽状态 ----
const leftWidth = ref(props.defaultLeft ?? MAX_LEFT);
const rightWidth = ref(props.defaultRight ?? MAX_RIGHT);
const leftCollapsed = ref(false);
const rightCollapsed = ref(false);

// 拖拽状态
let dragging: "left" | "right" | null = null;
let startX = 0;
let startWidth = 0;

function onMouseDown(side: "left" | "right", e: MouseEvent) {
  e.preventDefault();
  dragging = side;
  startX = e.clientX;
  startWidth = side === "left" ? leftWidth.value : rightWidth.value;
  document.addEventListener("mousemove", onMouseMove);
  document.addEventListener("mouseup", onMouseUp);
  document.body.style.cursor = "col-resize";
  document.body.style.userSelect = "none";
}

function onMouseMove(e: MouseEvent) {
  if (!dragging) return;
  const delta = e.clientX - startX;
  if (dragging === "left") {
    leftWidth.value = Math.max(MIN_WIDTH, Math.min(MAX_LEFT, startWidth + delta));
  } else {
    rightWidth.value = Math.max(MIN_WIDTH, Math.min(MAX_RIGHT, startWidth - delta));
  }
}

function onMouseUp() {
  if (dragging) {
    persistWidths();
    dragging = null;
  }
  document.removeEventListener("mousemove", onMouseMove);
  document.removeEventListener("mouseup", onMouseUp);
  document.body.style.cursor = "";
  document.body.style.userSelect = "";
}

async function persistWidths() {
  try {
    await Promise.all([
      settingsApi.setSetting("layout_left_width", String(leftWidth.value)),
      settingsApi.setSetting("layout_right_width", String(rightWidth.value)),
    ]);
  } catch {
    // 持久化失败静默忽略
  }
}

async function loadWidths() {
  try {
    const [lw, rw] = await Promise.all([
      settingsApi.getSetting("layout_left_width"),
      settingsApi.getSetting("layout_right_width"),
    ]);
    // 校验宽度值，拒绝异常小的值（可能是折叠时误保存的）
    if (lw) {
      const v = Number(lw);
      if (v >= MIN_WIDTH) leftWidth.value = v;
    }
    if (rw) {
      const v = Number(rw);
      if (v >= MIN_WIDTH) rightWidth.value = v;
    }
  } catch {
    // 首次加载无设置，使用默认值
  }
}

function toggleLeft() {
  leftCollapsed.value = !leftCollapsed.value;
}

function toggleRight() {
  rightCollapsed.value = !rightCollapsed.value;
}

onMounted(loadWidths);
</script>

<template>
  <div class="three-column-layout flex h-full w-full overflow-hidden">
    <!-- 左栏：章节树 -->
    <div
      class="left-panel flex flex-col border-r border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-850 transition-all duration-200"
      :style="{ width: leftCollapsed ? '0px' : `${leftWidth}px`, minWidth: leftCollapsed ? '0px' : `${MIN_WIDTH}px`, overflow: leftCollapsed ? 'hidden' : 'auto' }"
    >
      <slot name="left" />
    </div>

    <!-- 左分隔条 -->
    <div
      v-if="!leftCollapsed"
      class="divider resize-bar w-1 cursor-col-resize hover:bg-linden-primary/30 active:bg-linden-primary/50 transition-colors flex-shrink-0"
      @mousedown="onMouseDown('left', $event)"
    />

    <!-- 左折叠按钮 -->
    <NTooltip :delay="500" placement="right">
      <template #trigger>
        <div class="collapse-btn-wrap flex-shrink-0 w-5 flex items-center justify-center bg-gray-50 dark:bg-gray-800 hover:bg-gray-200 dark:hover:bg-gray-700 transition-colors cursor-pointer" @click="toggleLeft">
          <span v-if="leftCollapsed" class="i-carbon-chevron-right text-xs text-gray-500 dark:text-gray-400" />
          <span v-else class="i-carbon-chevron-left text-xs text-gray-500 dark:text-gray-400" />
        </div>
      </template>
      {{ leftCollapsed ? "展开章节树" : "收起章节树" }}
    </NTooltip>

    <!-- 中栏：编辑区 -->
    <div class="center-panel flex-1 flex flex-col min-w-0 overflow-hidden bg-white dark:bg-gray-900">
      <slot name="center" />
    </div>

    <!-- 右折叠按钮 -->
    <NTooltip :delay="500" placement="left">
      <template #trigger>
        <div class="collapse-btn-wrap flex-shrink-0 w-5 flex items-center justify-center bg-gray-50 dark:bg-gray-800 hover:bg-gray-200 dark:hover:bg-gray-700 transition-colors cursor-pointer" @click="toggleRight">
          <span v-if="rightCollapsed" class="i-carbon-chevron-left text-xs text-gray-500 dark:text-gray-400" />
          <span v-else class="i-carbon-chevron-right text-xs text-gray-500 dark:text-gray-400" />
        </div>
      </template>
      {{ rightCollapsed ? "展开侧栏" : "收起侧栏" }}
    </NTooltip>

    <!-- 右分隔条 -->
    <div
      v-if="!rightCollapsed"
      class="divider resize-bar w-1 cursor-col-resize hover:bg-linden-primary/30 active:bg-linden-primary/50 transition-colors flex-shrink-0"
      @mousedown="onMouseDown('right', $event)"
    />

    <!-- 右栏：信息/故事线/人物 -->
    <div
      class="right-panel flex flex-col border-l border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-850 transition-all duration-200"
      :style="{ width: rightCollapsed ? '0px' : `${rightWidth}px`, minWidth: rightCollapsed ? '0px' : `${MIN_WIDTH}px`, overflow: rightCollapsed ? 'hidden' : 'auto' }"
    >
      <slot name="right" />
    </div>
  </div>
</template>

<style scoped>
.collapse-btn-wrap {
  height: 100%;
}
.resize-bar {
  position: relative;
  z-index: 1;
}
.resize-bar::after {
  content: "";
  position: absolute;
  inset: 0 -3px;
}
</style>
