<script setup lang="ts">
import { computed, watch } from "vue";
import {
  NDrawer,
  NDrawerContent,
  NEmpty,
  NProgress,
  NTag,
  NButton,
  NScrollbar,
  NSpin,
  useMessage,
} from "naive-ui";
import {
  useTaskCenter,
  taskTypeLabel,
  taskStatusLabel,
  taskStatusTagType,
} from "../../composables/useTaskCenter";
import type { AsyncTask } from "../../api/tasks";
import { formatRelativeTime } from "../../utils/time";

const props = defineProps<{
  show: boolean;
  projectId: string;
}>();

const emit = defineEmits<{
  (e: "update:show", value: boolean): void;
}>();

const message = useMessage();
const {
  tasks,
  loading,
  activeTasks,
  recentTasks,
  refresh,
  cancelTask,
} = useTaskCenter();

// 打开抽屉时刷新一次
watch(
  () => props.show,
  (v) => {
    if (v && props.projectId) refresh(props.projectId);
  }
);

// 注意：关闭由 NDrawer 的 update:show 双向绑定自动处理，无需 handleClose

async function handleCancel(task: AsyncTask) {
  try {
    await cancelTask(task.id);
    message.success(`已取消任务：${taskTypeLabel(task.task_type)}`);
  } catch (e: any) {
    message.error(e?.toString() ?? "取消失败");
  }
}

/** 计算任务进度百分比（0-100），无 total 时返回 0 */
function progressPercent(task: AsyncTask): number {
  if (task.progress_total <= 0) return 0;
  return Math.min(
    100,
    Math.round((task.progress_current / task.progress_total) * 100)
  );
}

/** 任务的状态色（用于进度条） */
function progressStatus(
  task: AsyncTask
): "default" | "success" | "error" | "warning" | "info" {
  switch (task.status) {
    case "completed":
      return "success";
    case "failed":
      return "error";
    case "cancelled":
      return "warning";
    case "running":
      return "info";
    default:
      return "default";
  }
}

/** 任务条目显示名 */
function taskTitle(task: AsyncTask): string {
  const base = taskTypeLabel(task.task_type);
  // 优先用 target_type 让用户能识别是哪个元素/章节
  if (task.target_type) {
    const typeMap: Record<string, string> = {
      character: "角色",
      storyline: "故事线",
      worldview: "世界观",
      chapter: "章节",
    };
    const label = typeMap[task.target_type] ?? task.target_type;
    return `${base} · ${label}`;
  }
  return base;
}

const hasAny = computed(() => tasks.value.length > 0);
</script>

<template>
  <NDrawer
    :show="show"
    :width="420"
    placement="right"
    @update:show="(v) => emit('update:show', v)"
  >
    <NDrawerContent
      title="任务中心"
      :native-scrollbar="false"
      closable
    >
      <template #header>
        <div class="flex items-center justify-between w-full">
          <span class="text-base font-semibold text-gray-800 dark:text-gray-100">任务中心</span>
          <!-- <NTooltip trigger="hover">
            <template #trigger> -->
              <NButton
                size="tiny"
                quaternary
                circle
                @click="refresh(projectId)"
              >
                <span class="i-carbon-renew text-base" />
              </NButton>
            <!-- </template>
            刷新
          </NTooltip> -->
        </div>
      </template>

      <NSpin :show="loading">
        <div v-if="!hasAny && !loading" class="py-12">
          <NEmpty description="暂无任务" />
        </div>

        <div v-else class="flex flex-col gap-6 pb-4">
          <!-- 活跃任务 -->
          <section v-if="activeTasks.length > 0">
            <h3
              class="text-xs font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400 mb-2"
            >
              进行中 ({{ activeTasks.length }})
            </h3>
            <NScrollbar max-height="280">
              <div class="flex flex-col gap-2 pr-2">
                <div
                  v-for="task in activeTasks"
                  :key="task.id"
                  class="border border-gray-200 dark:border-gray-700 rounded-md p-2.5 bg-gray-50 dark:bg-gray-900/40"
                >
                  <div class="flex items-center justify-between mb-1.5">
                    <span class="text-sm font-medium truncate text-gray-800 dark:text-gray-200">
                      {{ taskTitle(task) }}
                    </span>
                  </div>

                  <NProgress
                    v-if="task.status === 'running'"
                    type="line"
                    :percentage="progressPercent(task)"
                    :status="progressStatus(task)"
                    :show-indicator="false"
                    :height="6"
                  />

                  <div
                    class="flex items-center justify-between mt-1.5 text-xs text-gray-500 dark:text-gray-400"
                  >
                    <div class="flex items-center gap-2">
                      <NTag
                        size="tiny"
                        :type="taskStatusTagType(task.status)"
                        round
                      >
                        {{ taskStatusLabel(task.status) }}
                      </NTag>
                      <span>{{ formatRelativeTime(task.started_at || task.created_at) }}</span>
                    </div>
                    <NButton
                      v-if="task.status === 'pending' || task.status === 'running'"
                      size="tiny"
                      quaternary
                      type="error"
                      @click="handleCancel(task)"
                    >
                      取消
                    </NButton>
                  </div>
                </div>
              </div>
            </NScrollbar>
          </section>

          <!-- 最近完成 -->
          <section v-if="recentTasks.length > 0">
            <h3
              class="text-xs font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400 mb-2"
            >
              最近完成 ({{ recentTasks.length }})
            </h3>
            <NScrollbar style="max-height: 360px">
              <div class="flex flex-col gap-1.5 pr-2">
                <div
                  v-for="task in recentTasks"
                  :key="task.id"
                  class="flex items-center justify-between text-xs border-b border-gray-100 dark:border-gray-800 py-1.5"
                >
                    <div class="flex flex-col min-w-0 flex-1">
                      <span class="truncate text-gray-700 dark:text-gray-300">{{ taskTitle(task) }}</span>
                      <span
                        v-if="task.error_message"
                        class="text-red-500 dark:text-red-400 truncate text-[11px]"
                        :title="task.error_message"
                      >
                        {{ task.error_message }}
                      </span>
                    </div>
                    <div class="flex items-center gap-3 ml-3 shrink-0">
                      <NTag
                        size="tiny"
                        :type="taskStatusTagType(task.status)"
                        round
                      >
                        {{ taskStatusLabel(task.status) }}
                      </NTag>
                      <span class="text-gray-400 dark:text-gray-500">
                        {{ formatRelativeTime(task.completed_at || task.created_at) }}
                      </span>
                    </div>
                  </div>
              </div>
            </NScrollbar>
          </section>
        </div>
      </NSpin>
    </NDrawerContent>
  </NDrawer>
</template>
