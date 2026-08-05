<script setup lang="ts">
import { computed } from "vue";
import { NTabs, NTabPane, NEmpty, NDescriptions, NDescriptionsItem, NTag } from "naive-ui";
import { useChapterStore } from "../../stores/chapter";

const chapterStore = useChapterStore();

const activeChapter = computed(() => {
  if (!chapterStore.activeChapterId) return null;
  return chapterStore.chapters.find((c) => c.id === chapterStore.activeChapterId) ?? null;
});

const statusMap: Record<string, { label: string; type: "default" | "info" | "success" | "warning" }> = {
  draft: { label: "草稿", type: "default" },
  writing: { label: "写作中", type: "info" },
  final: { label: "定稿", type: "success" },
};

function formatTime(iso: string | null) {
  if (!iso) return "-";
  const d = new Date(iso);
  return d.toLocaleString("zh-CN", {
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  });
}
</script>

<template>
  <div class="right-sidebar flex flex-col h-full text-gray-800 dark:text-gray-200">
    <NTabs type="line" size="small" class="flex-1 flex flex-col" default-value="info">
      <!-- Tab 1: 本章信息 -->
      <NTabPane name="info" tab="本章信息" class="flex-1 overflow-auto px-3 py-2">
        <NEmpty v-if="!activeChapter" description="选择一个章节查看信息" class="py-10" size="small" />
        <NDescriptions v-else label-placement="left" :column="1" size="small" bordered>
          <NDescriptionsItem label="章节名">{{ activeChapter.title }}</NDescriptionsItem>
          <NDescriptionsItem label="状态">
            <NTag :type="statusMap[activeChapter.status]?.type ?? 'default'" size="small">
              {{ statusMap[activeChapter.status]?.label ?? activeChapter.status }}
            </NTag>
          </NDescriptionsItem>
          <NDescriptionsItem label="字数">{{ chapterStore.wordCount }}</NDescriptionsItem>
          <NDescriptionsItem label="创建时间">{{ formatTime(activeChapter.created_at) }}</NDescriptionsItem>
          <NDescriptionsItem label="修改时间">{{ formatTime(activeChapter.updated_at) }}</NDescriptionsItem>
          <NDescriptionsItem label="摘要">
            <span class="text-xs text-gray-500 dark:text-gray-400">{{ activeChapter.summary || "暂无" }}</span>
          </NDescriptionsItem>
        </NDescriptions>
      </NTabPane>

      <!-- Tab 2: 故事线 -->
      <NTabPane name="storyline" tab="故事线" class="flex-1 overflow-auto px-3 py-2">
        <NEmpty description="故事线功能将在后续版本中实现" class="py-10" size="small">
          <template #icon>
            <span class="i-carbon-branch text-2xl opacity-30" />
          </template>
        </NEmpty>
      </NTabPane>

      <!-- Tab 3: 人物 -->
      <NTabPane name="characters" tab="人物" class="flex-1 overflow-auto px-3 py-2">
        <NEmpty description="人物肖像功能将在后续版本中实现" class="py-10" size="small">
          <template #icon>
            <span class="i-carbon-user-avatar text-2xl opacity-30" />
          </template>
        </NEmpty>
      </NTabPane>

      <!-- Tab 4: 世界观 -->
      <NTabPane name="worldview" tab="世界观" class="flex-1 overflow-auto px-3 py-2">
        <NEmpty description="世界观功能将在后续版本中实现" class="py-10" size="small">
          <template #icon>
            <span class="i-carbon-earth text-2xl opacity-30" />
          </template>
        </NEmpty>
      </NTabPane>
    </NTabs>
  </div>
</template>
