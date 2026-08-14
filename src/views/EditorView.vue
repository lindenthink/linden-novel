<script setup lang="ts">
import { onMounted, onUnmounted, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { save } from "@tauri-apps/plugin-dialog";
import { NButton, NSpace, NDropdown, NEmpty, useMessage, useDialog } from "naive-ui";
import { useProjectStore } from "../stores/project";
import { useChapterStore } from "../stores/chapter";
import { useLongContext } from "../composables/useLongContext";
import { useEditorUI } from "../composables/useEditorUI";
import { useTaskCenter } from "../composables/useTaskCenter";
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
const dialog = useDialog();
const { openAIGeneration } = useEditorUI();
const {
  summaryLoading,
  batchLoading,
  embeddingLoading,
  handleGenerateSummary,
  handleBatchSummaries,
  handleSyncEmbeddings,
} = useLongContext();
const { init: initTaskCenter, cleanup: cleanupTaskCenter } = useTaskCenter();

const projectId = route.params.id as string;

async function loadProject() {
  try {
    chapterStore.clearChapters();
    await projectStore.selectProject(projectId);
    // 加载所有卷的章节
    for (const vol of projectStore.volumes) {
      await chapterStore.fetchChapters(vol.id);
    }
    // 恢复上次编辑的章节
    const lastChapterId = localStorage.getItem(`linden:lastChapter:${projectId}`);
    if (lastChapterId && chapterStore.chapters.some((c) => c.id === lastChapterId)) {
      await chapterStore.setActiveChapter(lastChapterId);
    }
    // 初始化任务中心：拉取历史任务 + 启动事件监听
    await initTaskCenter(projectId);
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

// ---- AI 生成 ----
function handleOpenAIGeneration() {
  if (!chapterStore.activeChapterId) {
    message.warning("请先选择一个章节");
    return;
  }
  openAIGeneration("continuation");
}

// ---- 长上下文操作 ----
async function generateSummaryForCurrent() {
  const chapterId = chapterStore.activeChapterId;
  if (!chapterId) {
    message.warning("请先选择一个章节");
    return;
  }

  // 检查章节内容是否为空
  const contentText = chapterStore.activeContent?.content_text?.trim();
  if (!contentText) {
    message.warning("当前章节内容为空，请先编写章节正文后再生成摘要。");
    return;
  }

  // 从章节数据中检查是否已有摘要
  const currentChapter = chapterStore.chapters.find((c) => c.id === chapterId);
  const existingSummary = currentChapter?.summary;
  
  const doGenerate = async () => {
    try {
      const res = await handleGenerateSummary(chapterId);
      // 更新 store 中的章节 summary
      if (currentChapter) {
        chapterStore.updateChapterMeta(chapterId, { summary: res.summary });
      }
      message.success(`摘要已生成（${res.char_count} 字）`);
    } catch (e: any) {
      message.error(e?.toString() || "摘要生成失败");
    }
  };

  if (existingSummary && existingSummary.trim().length > 0) {
    dialog.warning({
      title: "确认重新生成",
      content: "当前章节已有摘要，重新生成将覆盖原有摘要，确定继续吗？",
      positiveText: "重新生成",
      negativeText: "取消",
      onPositiveClick: doGenerate,
    });
    return;
  }

  await doGenerate();
}

async function batchGenerateAllSummaries() {
  dialog.warning({
    title: "确认批量生成",
    content:
      "将为项目内所有章节逐一调用 AI 生成摘要，可能耗时较长且消耗较多 Token，确定继续吗？",
    positiveText: "继续生成",
    negativeText: "取消",
    onPositiveClick: async () => {
      try {
        await handleBatchSummaries(projectId);
        message.success("批量摘要任务已提交，请在任务中心查看进度");
      } catch (e: any) {
        message.error(e?.toString() || "批量摘要生成失败");
      }
    },
  });
}

async function syncAllEmbeddings() {
  try {
    await handleSyncEmbeddings(projectId);
    message.success("嵌入同步任务已提交，请在任务中心查看进度");
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

onUnmounted(() => {
  // 离开编辑器时清理任务中心监听
  cleanupTaskCenter();
});

// 路由变化时重新加载
watch(() => route.params.id, (newId) => {
  if (newId && newId !== projectId) {
    loadProject();
  }
});

// 记忆上次编辑的章节
watch(() => chapterStore.activeChapterId, (chId) => {
  if (chId) {
    localStorage.setItem(`linden:lastChapter:${projectId}`, chId);
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
        <NButton quaternary size="small" @click="handleOpenAIGeneration">
          <template #icon>
            <span class="i-carbon-ai-generate" />
          </template>
          AI 生成
        </NButton>
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
          <!-- 空项目：引导新建卷和章节 -->
          <div
            v-if="projectStore.volumes.length === 0"
            class="flex items-center justify-center h-full bg-gray-50 dark:bg-gray-900"
          >
            <NEmpty size="large" description="还没有卷和章节" class="py-10">
              <template #extra>
                <NSpace vertical align="center" :size="12">
                  <span class="text-sm text-gray-500 dark:text-gray-400">
                    在左侧章节目录点击「+ 卷」创建第一卷
                  </span>
                  <span class="text-xs text-gray-400 dark:text-gray-500">
                    创建卷后，右键点击卷可新建章节
                  </span>
                </NSpace>
              </template>
            </NEmpty>
          </div>
          <!-- 有章节但未选中：提示选择 -->
          <div
            v-else-if="!chapterStore.activeChapterId"
            class="flex items-center justify-center h-full bg-gray-50 dark:bg-gray-900"
          >
            <NEmpty size="large" description="请从左侧选择一个章节开始写作" class="py-10" />
          </div>
          <!-- 编辑区 — TipTap 编辑器 -->
          <LindenEditor v-else />
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
