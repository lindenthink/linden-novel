<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { save } from "@tauri-apps/plugin-dialog";
import { NButton, NSpace, NDropdown, useMessage } from "naive-ui";
import { useProjectStore } from "../stores/project";
import { useChapterStore } from "../stores/chapter";
import { useLongContext } from "../composables/useLongContext";
import { exportProject } from "../api/io";
import ThreeColumnLayout from "../components/layout/ThreeColumnLayout.vue";
import ChapterTree from "../components/layout/ChapterTree.vue";
import RightSidebar from "../components/layout/RightSidebar.vue";
import StatusBar from "../components/layout/StatusBar.vue";
import LindenEditor from "../components/editor/LindenEditor.vue";

const route = useRoute();
const router = useRouter();
const projectStore = useProjectStore();
const chapterStore = useChapterStore();
const message = useMessage();
const {
  summaryLoading,
  batchLoading,
  embeddingLoading,
  handleGenerateSummary,
  handleBatchSummaries,
  handleSyncEmbeddings,
} = useLongContext();

const projectId = route.params.id as string;

async function loadProject() {
  try {
    chapterStore.clearChapters();
    await projectStore.selectProject(projectId);
    // 加载所有卷的章节
    for (const vol of projectStore.volumes) {
      await chapterStore.fetchChapters(vol.id);
    }
  } catch (e: any) {
    message.error(e?.message || "加载项目失败");
    router.replace({ name: "home" });
  }
}

function goHome() {
  router.push({ name: "home" });
}

// ---- 导出 ----
const exporting = ref(false);
const exportOptions = [
  { label: "导出为 TXT", key: "txt" },
  { label: "导出为 Markdown", key: "md" },
  { label: "导出为 JSON", key: "json" },
];

async function handleExport(key: string) {
  try {
    const ext = key === "md" ? "md" : key;
    const defaultName = `${projectStore.currentProject?.title || "项目"}.${ext}`;
    const selected = await save({
      filters: [
        {
          name: key === "json" ? "JSON 文件" : key === "md" ? "Markdown 文件" : "文本文件",
          extensions: [ext],
        },
      ],
      defaultPath: defaultName,
      title: "导出项目",
    });
    if (!selected) return;

    exporting.value = true;
    await exportProject(projectId, key, selected);
    message.success("导出成功");
  } catch (e: any) {
    message.error(e?.message || "导出失败");
  } finally {
    exporting.value = false;
  }
}

// ---- 长上下文操作 ----
async function generateSummaryForCurrent() {
  const chapterId = chapterStore.activeChapterId;
  if (!chapterId) {
    message.warning("请先选择一个章节");
    return;
  }
  try {
    const res = await handleGenerateSummary(chapterId);
    message.success(`摘要已生成（${res.char_count} 字）`);
  } catch (e: any) {
    message.error(e?.toString() || "摘要生成失败");
  }
}

async function batchGenerateAllSummaries() {
  try {
    const res = await handleBatchSummaries(projectId);
    message.success(`批量完成：成功 ${res.success_count}，失败 ${res.failed_count}`);
  } catch (e: any) {
    message.error(e?.toString() || "批量摘要生成失败");
  }
}

async function syncAllEmbeddings() {
  try {
    const res = await handleSyncEmbeddings(projectId);
    message.success(`嵌入同步完成：${res.embedded_count} 个条目`);
  } catch (e: any) {
    message.error(e?.toString() || "嵌入同步失败");
  }
}

const longContextOptions = [
  {
    label: "为当前章节生成摘要",
    key: "summary-current",
    disabled: summaryLoading.value,
  },
  {
    label: "批量生成所有章节摘要",
    key: "summary-batch",
    disabled: batchLoading.value,
  },
  { type: "divider", key: "d1" },
  {
    label: "同步语义索引（嵌入）",
    key: "embedding-sync",
    disabled: embeddingLoading.value,
  },
];

function handleLongContextSelect(key: string) {
  switch (key) {
    case "summary-current":
      generateSummaryForCurrent();
      break;
    case "summary-batch":
      batchGenerateAllSummaries();
      break;
    case "embedding-sync":
      syncAllEmbeddings();
      break;
  }
}

onMounted(loadProject);

// 路由变化时重新加载
watch(() => route.params.id, (newId) => {
  if (newId && newId !== projectId) {
    loadProject();
  }
});
</script>

<template>
  <div class="editor-view flex flex-col h-screen bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100 transition-colors">
    <!-- 顶部工具栏 -->
    <header
      class="h-10 flex items-center justify-between px-3 border-b border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 flex-shrink-0"
    >
      <div class="flex items-center gap-2">
        <NButton quaternary size="small" @click="goHome">
          <template #icon>
            <span class="i-carbon-chevron-left" />
          </template>
        </NButton>
        <span class="text-sm font-semibold truncate max-w-60 text-gray-800 dark:text-gray-200">
          {{ projectStore.currentProject?.title ?? "加载中..." }}
        </span>
      </div>
      <NSpace size="small" align="center">
        <NDropdown
          :options="longContextOptions"
          @select="handleLongContextSelect"
        >
          <NButton
            quaternary
            size="small"
            :loading="summaryLoading || batchLoading || embeddingLoading"
          >
            <template #icon>
              <span class="i-carbon-search-locate" />
            </template>
            上下文
          </NButton>
        </NDropdown>
        <NDropdown
          :options="exportOptions"
          @select="handleExport"
          :disabled="exporting"
        >
          <NButton quaternary size="small" :loading="exporting">
            <template #icon>
              <span class="i-carbon-export" />
            </template>
            导出
          </NButton>
        </NDropdown>
      </NSpace>
    </header>

    <!-- 三栏布局 -->
    <div class="flex-1 min-h-0">
      <ThreeColumnLayout>
        <template #left>
          <ChapterTree />
        </template>
        <template #center>
          <!-- 编辑区 — TipTap 编辑器 -->
          <LindenEditor />
        </template>
        <template #right>
          <RightSidebar />
        </template>
      </ThreeColumnLayout>
    </div>

    <!-- 状态栏 -->
    <StatusBar />
  </div>
</template>
