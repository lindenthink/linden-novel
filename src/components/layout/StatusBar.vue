<script setup lang="ts">
import { computed } from "vue";
import { useChapterStore } from "../../stores/chapter";
import { useProjectStore } from "../../stores/project";
import { useWordCount } from "../../composables/useWordCount";
import { useEditorSettings } from "../../composables/useEditorSettings";
import { NSwitch, NTooltip, useMessage } from "naive-ui";

const chapterStore = useChapterStore();
const projectStore = useProjectStore();
const { autoSaveEnabled, setAutoSave } = useEditorSettings();
const message = useMessage();

// 实时字数统计（前端估值）：传入当前正文文本的 ref
const contentText = computed(() => chapterStore.activeContent?.content_text ?? "");
const { wordCount: liveWordCount } = useWordCount(contentText);

const activeChapter = computed(() => {
  if (!chapterStore.activeChapterId) return null;
  return chapterStore.chapters.find((c) => c.id === chapterStore.activeChapterId) ?? null;
});

const statusLabel = computed(() => {
  if (!activeChapter.value) return "";
  const map: Record<string, string> = { draft: "草稿", writing: "写作中", final: "定稿" };
  return map[activeChapter.value.status] ?? activeChapter.value.status;
});

const saveLabel = computed(() => {
  if (chapterStore.saving) return "保存中...";
  if (chapterStore.dirty) return "未保存";
  return "已保存";
});

async function handleAutoSaveChange(checked: boolean) {
  await setAutoSave(checked);
  message.success(checked ? "已启用自动保存" : "已关闭自动保存");
}
</script>

<template>
  <footer
    class="status-bar h-6 flex items-center gap-4 px-3 text-xs border-t border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-800 text-gray-600 dark:text-gray-400 select-none"
  >
    <!-- 项目名 -->
    <span v-if="projectStore.currentProject" class="truncate max-w-48">
      {{ projectStore.currentProject.title }}
    </span>

    <span class="opacity-30">|</span>

    <!-- 章节信息 -->
    <span v-if="activeChapter" class="truncate">
      {{ activeChapter.title }}
    </span>
    <span v-else class="opacity-50">未选择章节</span>

    <span class="opacity-30">|</span>

    <!-- 字数：优先显示实时统计，保存后使用权威值 -->
    <span>{{ chapterStore.dirty ? liveWordCount : chapterStore.wordCount }} 字</span>

    <span class="opacity-30">|</span>

    <!-- 状态 -->
    <span v-if="statusLabel">{{ statusLabel }}</span>

    <!-- 右侧：自动保存开关 + 保存状态 -->
    <div class="ml-auto flex items-center gap-3">
      <NTooltip :trigger="'hover'" placement="top">
        <template #trigger>
          <div class="flex items-center gap-1 cursor-pointer">
            <span :class="autoSaveEnabled ? 'text-emerald-500' : 'text-gray-400'"
              :style="{ fontSize: '12px', lineHeight: 1 }">
              <span class="i-carbon-save" />
            </span>
            <NSwitch
              :value="autoSaveEnabled"
              size="small"
              @update:value="handleAutoSaveChange"
            />
          </div>
        </template>
        {{ autoSaveEnabled ? "自动保存：开" : "自动保存：关" }}
      </NTooltip>

      <span :class="chapterStore.dirty ? 'text-amber-500' : 'opacity-50'">
        {{ saveLabel }}
      </span>
    </div>
  </footer>
</template>
