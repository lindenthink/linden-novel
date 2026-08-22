<script setup lang="ts">
import { NCard, NThing, NDropdown, NButton, NEllipsis, NTag } from "naive-ui";
import type { Project } from "../../types";
import { formatLocalTime } from "../../utils/time";

const props = defineProps<{
  project: Project;
}>();

const emit = defineEmits<{
  open: [id: string];
  delete: [id: string];
  edit: [id: string];
}>();

const statusLabel = (p: Project) => {
  if (p.genre) return p.genre;
  return "未分类";
};

const dropdownOptions = [
  { label: "编辑信息", key: "edit" },
  { label: "删除项目", key: "delete" },
];

function handleDropdown(key: string) {
  if (key === "edit") emit("edit", props.project.id);
  if (key === "delete") emit("delete", props.project.id);
}

const formatTime = (iso: string) =>
  formatLocalTime(iso, {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  });
</script>

<template>
  <NCard
    hoverable
    class="cursor-pointer transition-all duration-200 hover:shadow-lg bg-white dark:bg-gray-800"
    @click="emit('open', project.id)"
  >
    <div class="flex items-start justify-between">
      <NThing>
        <template #header>
          <span class="text-base font-semibold text-gray-900 dark:text-gray-100">{{ project.title }}</span>
        </template>
        <template #description>
          <NEllipsis :line-clamp="2" class="text-sm text-gray-500 dark:text-gray-400">
            {{ project.summary || "暂无简介" }}
          </NEllipsis>
        </template>
      </NThing>
      <NDropdown
        :options="dropdownOptions"
        trigger="click"
        @select="handleDropdown"
        @click.stop
      >
        <NButton quaternary size="small" @click.stop>
          <template #icon>
            <span class="i-carbon-overflow-menu-horizontal" />
          </template>
        </NButton>
      </NDropdown>
      
    </div>

    <div class="mt-3 flex items-center gap-2 text-xs text-gray-500 dark:text-gray-400">
      <NTag size="small" :bordered="false" type="info">
        {{ statusLabel(project) }}
      </NTag>
      <span v-if="project.target_words">
        目标 {{ (project.target_words / 10000).toFixed(1) }} 万字
      </span>
      <span class="ml-auto">{{ formatTime(project.updated_at) }}</span>
    </div>
  </NCard>
</template>
