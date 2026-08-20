<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { NModal, NButton, NProgress, NSpin, NScrollbar } from "naive-ui";
import { useUpdater } from "../../composables/useUpdater";

const props = defineProps<{ show: boolean }>();
const emit = defineEmits<{ (e: "update:show", v: boolean): void }>();

const {
  state,
  updateInfo,
  progress,
  errorMsg,
  checkForUpdates,
  downloadAndInstall,
  relaunch,
  reset,
} = useUpdater();

const currentVersion = ref("");

async function loadCurrentVersion() {
  try {
    const { getVersion } = await import("@tauri-apps/api/app");
    currentVersion.value = await getVersion();
  } catch (e) {
    console.warn("[updater] get current version failed:", e);
  }
}

// 打开时拉一次当前版本（手动入口；自动检查走 App.vue，但 dialog 仍可能复用）
watch(
  () => props.show,
  (v) => {
    if (v && !currentVersion.value) loadCurrentVersion();
  },
  { immediate: true },
);

// 关闭逻辑：仅在 idle/not-available/error 状态可关闭
const closable = computed(
  () =>
    state.value === "idle" ||
    state.value === "not-available" ||
    state.value === "error",
);

function handleClose(v: boolean) {
  if (!v) {
    // 用户点了关闭/遮罩：若处于可中断状态则重置内部状态
    if (closable.value) {
      reset();
      loadCurrentVersion();
    }
  }
  emit("update:show", v);
}

async function handleDownload() {
  await downloadAndInstall();
}

async function handleRelaunch() {
  await relaunch();
}

async function handleRetry() {
  reset();
  await checkForUpdates(false);
}
</script>

<template>
  <NModal
    :show="show"
    @update:show="handleClose"
    :mask-closable="true"
    :close-on-esc="closable"
    :closable="closable"
    preset="card"
    title="检查更新"
    style="width: 480px; max-width: 90vw"
    :bordered="false"
  >
    <div class="py-2">
      <!-- 检查中 -->
      <div
        v-if="state === 'checking'"
        class="flex flex-col items-center justify-center gap-3 py-6"
      >
        <NSpin size="medium" />
        <p class="text-gray-600 dark:text-gray-300">正在检查更新...</p>
      </div>

      <!-- 有新版可更新 -->
      <div v-else-if="state === 'available'" class="flex flex-col gap-3">
        <div class="flex items-center gap-2">
          <span class="i-carbon-cloud-download text-xl text-primary" />
          <span class="text-base font-medium">
            发现新版本
            <span class="text-primary">v{{ updateInfo?.version }}</span>
          </span>
        </div>
        <div class="text-sm text-gray-500 dark:text-gray-400">
          当前版本：v{{ currentVersion || "未知" }}
        </div>
        <div
          v-if="updateInfo?.notes"
          class="border border-gray-200 dark:border-gray-700 rounded p-3 bg-gray-50 dark:bg-gray-800/50"
        >
          <div class="text-xs text-gray-500 dark:text-gray-400 mb-1">
            更新内容
          </div>
          <NScrollbar style="max-height: 200px">
            <pre
              class="text-sm whitespace-pre-wrap break-words text-gray-700 dark:text-gray-200 m-0"
              >{{ updateInfo.notes }}</pre
            >
          </NScrollbar>
        </div>
        <div class="flex justify-end gap-2 mt-2">
          <NButton @click="handleClose(false)">稍后再说</NButton>
          <NButton type="primary" @click="handleDownload">
            立即下载安装
          </NButton>
        </div>
      </div>

      <!-- 下载中 -->
      <div
        v-else-if="state === 'downloading'"
        class="flex flex-col gap-4 py-2"
      >
        <div class="flex items-center gap-2">
          <span class="i-carbon-cloud-download text-xl text-primary" />
          <span class="text-base font-medium">正在下载新版本...</span>
        </div>
        <NProgress
          type="line"
          :percentage="progress"
          :show-indicator="true"
          status="success"
        />
        <p class="text-sm text-gray-500 dark:text-gray-400">
          下载进度 {{ progress }}%，请勿关闭应用
        </p>
      </div>

      <!-- 已安装 -->
      <div
        v-else-if="state === 'installed'"
        class="flex flex-col items-center justify-center gap-3 py-4"
      >
        <span class="i-carbon-checkmark-filled text-3xl text-green-500" />
        <p class="text-base font-medium">更新已就绪</p>
        <p class="text-sm text-gray-500 dark:text-gray-400 text-center">
          重启应用后将进入 v{{ updateInfo?.version }}
        </p>
        <NButton type="primary" @click="handleRelaunch" class="mt-2">
          立即重启应用
        </NButton>
      </div>

      <!-- 已是最新 -->
      <div
        v-else-if="state === 'not-available'"
        class="flex flex-col items-center justify-center gap-3 py-6"
      >
        <span class="i-carbon-checkmark-filled text-3xl text-green-500" />
        <p class="text-base font-medium">当前已是最新版本</p>
        <p class="text-sm text-gray-500 dark:text-gray-400">
          v{{ currentVersion || "未知" }}
        </p>
      </div>

      <!-- 错误 -->
      <div
        v-else-if="state === 'error'"
        class="flex flex-col items-center justify-center gap-3 py-6"
      >
        <span class="i-carbon-warning-alt text-3xl text-amber-500" />
        <p class="text-base font-medium">检查更新失败</p>
        <p
          class="text-sm text-gray-500 dark:text-gray-400 text-center break-words max-w-full"
        >
          {{ errorMsg }}
        </p>
        <div class="flex gap-2 mt-2">
          <NButton @click="handleClose(false)">关闭</NButton>
          <NButton type="primary" @click="handleRetry">重试</NButton>
        </div>
      </div>

      <!-- idle -->
      <div v-else class="flex justify-center py-6">
        <NButton type="primary" @click="handleRetry">检查更新</NButton>
      </div>
    </div>
  </NModal>
</template>
