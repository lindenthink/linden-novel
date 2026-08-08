<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { NTag, NButton, NDropdown, useMessage } from "naive-ui";
import { useChapterStore } from "../../stores/chapter";
import { useElementStore } from "../../stores/element";
import { useProjectStore } from "../../stores/project";
import type { ElementType } from "../../types";

const chapterStore = useChapterStore();
const elementStore = useElementStore();
const projectStore = useProjectStore();
const message = useMessage();

const showAddDropdown = ref(false);

// 当前章节 ID
const currentChapterId = computed(() => chapterStore.activeChapterId);

// 当前章节关联的元素
const linkedElements = computed(() => {
  if (!currentChapterId.value) return [];
  return elementStore.chapterElements;
});

// 获取元素名称
function getElementName(elementType: string, elementId: string): string {
  const list =
    elementType === "character"
      ? elementStore.characters
      : elementType === "storyline"
        ? elementStore.storylines
        : elementStore.worldview;
  const item = list.find((e) => e.id === elementId);
  return item?.name ?? "未知元素";
}

// 获取元素类型标签
function getTypeLabel(type: string): string {
  return type === "character" ? "人物" : type === "storyline" ? "故事线" : "世界观";
}

// 获取元素类型颜色
function getTypeColor(type: string): "success" | "info" | "warning" {
  return type === "character" ? "success" : type === "storyline" ? "info" : "warning";
}

// 添加元素下拉选项
const addOptions = computed(() => {
  if (!currentChapterId.value) return [];

  const linked = new Set(
    linkedElements.value.map((ce) => `${ce.element_type}:${ce.element_id}`),
  );

  const options: { label: string; key: string; type?: "group" }[] = [];

  // 人物
  const availableChars = elementStore.characters.filter(
    (c) => !linked.has(`character:${c.id}`),
  );
  if (availableChars.length > 0) {
    options.push({ label: "人物", key: "group-character", type: "group" });
    availableChars.forEach((c) => {
      options.push({
        label: c.name,
        key: `character:${c.id}`,
      });
    });
  }

  // 故事线
  const availableStorylines = elementStore.storylines.filter(
    (s) => !linked.has(`storyline:${s.id}`),
  );
  if (availableStorylines.length > 0) {
    options.push({ label: "故事线", key: "group-storyline", type: "group" });
    availableStorylines.forEach((s) => {
      options.push({
        label: s.name,
        key: `storyline:${s.id}`,
      });
    });
  }

  // 世界观
  const availableWorldview = elementStore.worldview.filter(
    (w) => !linked.has(`worldview:${w.id}`),
  );
  if (availableWorldview.length > 0) {
    options.push({ label: "世界观", key: "group-worldview", type: "group" });
    availableWorldview.forEach((w) => {
      options.push({
        label: w.name,
        key: `worldview:${w.id}`,
      });
    });
  }

  return options;
});

// 加载当前章节的元素
watch(
  currentChapterId,
  async (id) => {
    if (id) {
      await elementStore.fetchChapterElements(id);
    }
  },
  { immediate: true },
);

// 确保元素名称数据已加载（人物、故事线、世界观）
// 这些数据原本只在右侧面板对应标签页点击时才加载，
// 但 ChapterElementBar 渲染标签时需要它们来显示名称
watch(
  () => projectStore.currentProject?.id,
  async (projectId) => {
    if (projectId) {
      await Promise.all([
        elementStore.fetchCharacters(projectId),
        elementStore.fetchStorylines(projectId),
        elementStore.fetchWorldview(projectId),
      ]);
    }
  },
  { immediate: true },
);

// 添加元素关联
async function handleAddElement(key: string) {
  if (!currentChapterId.value) return;

  const [elementType, elementId] = key.split(":");
  if (!elementType || !elementId) return;

  try {
    await elementStore.addChapterElement({
      chapter_id: currentChapterId.value,
      element_type: elementType as ElementType,
      element_id: elementId,
    });
    message.success("已添加关联");
  } catch (e: any) {
    message.error(e?.message || "添加关联失败");
  }
}

// 移除元素关联
async function handleRemoveElement(elementType: string, elementId: string) {
  if (!currentChapterId.value) return;

  try {
    await elementStore.removeChapterElementByRef(
      currentChapterId.value,
      elementType,
      elementId,
    );
    message.success("已移除关联");
  } catch (e: any) {
    message.error(e?.message || "移除关联失败");
  }
}
</script>

<template>
  <div
    v-if="currentChapterId"
    class="chapter-element-bar flex items-center gap-2 px-4 py-2 border-b border-gray-100 dark:border-gray-800 flex-shrink-0"
  >
    <span class="text-xs text-gray-400 dark:text-gray-500 flex-shrink-0">本章元素:</span>

    <!-- 已关联的元素标签 -->
    <div class="flex items-center gap-1.5 flex-wrap flex-1 min-w-0">
      <NTag
        v-for="ce in linkedElements"
        :key="ce.id"
        :type="getTypeColor(ce.element_type)"
        size="small"
        closable
        @close="handleRemoveElement(ce.element_type, ce.element_id)"
      >
        <span class="text-xs">
          {{ getTypeLabel(ce.element_type) }}: {{ getElementName(ce.element_type, ce.element_id) }}
        </span>
      </NTag>

      <span
        v-if="linkedElements.length === 0"
        class="text-xs text-gray-300 dark:text-gray-600"
      >
        暂无关联元素
      </span>
    </div>

    <!-- 添加元素按钮 -->
    <NDropdown
      v-model:show="showAddDropdown"
      trigger="click"
      :options="addOptions"
      @select="handleAddElement"
      :max-height="300"
    >
      <NButton size="tiny" quaternary :disabled="addOptions.length === 0">
        <template #icon>
          <span class="i-carbon-add" />
        </template>
        添加
      </NButton>
    </NDropdown>
  </div>
</template>
