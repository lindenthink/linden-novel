<script setup lang="ts">
import { computed, ref } from "vue";
import { useChapterStore } from "../../stores/chapter";
import { useProjectStore } from "../../stores/project";
import { useWordCount } from "../../composables/useWordCount";
import { useEditorSettings } from "../../composables/useEditorSettings";
import { useTaskCenter } from "../../composables/useTaskCenter";
import { NSwitch, NTooltip, NBadge, useMessage } from "naive-ui";
import TaskCenter from "../common/TaskCenter.vue";

const chapterStore = useChapterStore();
const projectStore = useProjectStore();
const { autoSaveEnabled, setAutoSave } = useEditorSettings();
const { activeCount } = useTaskCenter();
const message = useMessage();

const showTaskCenter = ref(false);

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

function openTaskCenter() {
  showTaskCenter.value = true;
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

    <!-- 右侧：任务中心入口 + 自动保存开关 + 保存状态 -->
    <div class="ml-auto flex items-center gap-3">
      <NTooltip :trigger="'hover'" placement="top">
        <template #trigger>
          <NBadge
            :value="activeCount"
            :max="99"
            :show="activeCount > 0"
            type="info"
            :offset="[-2, 2]"
          >
            <button
              class="task-center-btn flex items-center justify-center w-7 h-7 rounded-lg transition-all duration-300"
              :class="activeCount > 0
                ? 'text-blue-500 dark:text-blue-400 hover:text-blue-600 dark:hover:text-blue-500 hover:bg-blue-500/10 dark:hover:bg-blue-400/15'
                : 'text-gray-600 dark:text-gray-300 hover:text-emerald-600 dark:hover:text-emerald-500 hover:bg-emerald-500/10 dark:hover:bg-emerald-400/15'"
              @click="openTaskCenter"
              aria-label="任务中心"
            >
              <svg 
                :class="activeCount > 0 ? 'icon-spinning' : 'icon-resting'"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
              >
                <path d="M9 11l3 3L22 4" />
                <path d="M21 12v7a2 2 0 01-2 2H5a2 2 0 01-2-2V5a2 2 0 012-2h11" />
              </svg>
            </button>
          </NBadge>
        </template>
        任务中心{{ activeCount > 0 ? `（${activeCount} 个进行中）` : "" }}
      </NTooltip>

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

    <!-- 任务中心抽屉 -->
    <TaskCenter
      v-model:show="showTaskCenter"
      :project-id="projectStore.currentProject?.id ?? ''"
    />
  </footer>
</template>

<style>
.task-center-btn {
  position: relative;
  border: none;
  outline: none;
  background: transparent;
}

.task-center-btn:focus,
.task-center-btn:focus-visible {
  outline: none;
  box-shadow: none;
  border: none;
}

.task-center-btn svg {
  width: 18px;
  height: 18px;
}

.icon-spinning {
  animation: task-spin 1s cubic-bezier(0.4, 0, 0.2, 1) infinite;
}

.icon-resting {
  transition: transform 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

.task-center-btn:hover .icon-resting {
  transform: scale(1.08);
}

@keyframes task-spin {
  0% {
    transform: rotate(0deg);
  }
  100% {
    transform: rotate(360deg);
  }
}

/* NBadge 样式覆盖 */
.n-badge {
  background-color: transparent !important;
}

.n-badge .n-badge__icon,
.n-badge .n-badge__supplement {
  background-color: transparent !important;
}
</style>
