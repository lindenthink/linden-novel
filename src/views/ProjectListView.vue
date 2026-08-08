<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { open } from "@tauri-apps/plugin-dialog";
import {
  NButton,
  NGrid,
  NGi,
  NModal,
  NForm,
  NFormItem,
  NInput,
  NSelect,
  NInputNumber,
  useMessage,
  useDialog,
  NSpin,
  NEmpty,
} from "naive-ui";
import { useProjectStore } from "../stores/project";
import { importProject } from "../api/io";
import { useTheme } from "../composables/useTheme";
import AiSettingsDialog from "../components/ai/AiSettingsDialog.vue";
import ProjectCard from "../components/common/ProjectCard.vue";

const router = useRouter();
const projectStore = useProjectStore();
const message = useMessage();
const dialog = useDialog();
const { toggle, isDark } = useTheme();

// ---- AI 设置弹窗 ----
const showAiSettings = ref(false);

// ---- 新建项目弹窗 ----
const showCreate = ref(false);
const createForm = ref({
  title: "",
  genre: null as string | null,
  summary: null as string | null,
  target_words: null as number | null,
});
const creating = ref(false);

// ---- 编辑项目弹窗 ----
const showEdit = ref(false);
const editingId = ref<string | null>(null);
const editForm = ref({
  title: "",
  genre: null as string | null,
  summary: null as string | null,
  target_words: null as number | null,
});
const editing = ref(false);

const genreOptions = [
  { label: "玄幻", value: "玄幻" },
  { label: "修真", value: "修真" },
  { label: "网游", value: "网游" },
  { label: "都市", value: "都市" },
  { label: "言情", value: "言情" },
  { label: "悬疑", value: "悬疑" },
  { label: "武侠", value: "武侠" },
  { label: "科幻", value: "科幻" },
  { label: "历史", value: "历史" },
  { label: "文学", value: "文学" },
  { label: "其他", value: "其他" },
];

async function handleCreate() {
  if (!createForm.value.title.trim()) {
    message.warning("请输入项目名称");
    return;
  }
  creating.value = true;
  try {
    const p = await projectStore.createProject(createForm.value.title.trim());
    message.success("项目创建成功");
    showCreate.value = false;
    createForm.value = { title: "", genre: null, summary: null, target_words: null };
    router.push({ name: "editor", params: { id: p.id } });
  } catch (e: any) {
    message.error(e?.message || "创建失败");
  } finally {
    creating.value = false;
  }
}

function handleOpen(id: string) {
  router.push({ name: "editor", params: { id } });
}

function handleDelete(id: string) {
  const p = projectStore.projects.find((x) => x.id === id);
  dialog.warning({
    title: "确认删除",
    content: `确定要删除项目「${p?.title || ""}」吗？此操作不可撤销，所有卷和章节将被一并删除。`,
    positiveText: "删除",
    negativeText: "取消",
    onPositiveClick: async () => {
      try {
        await projectStore.deleteProject(id);
        message.success("已删除");
      } catch (e: any) {
        message.error(e?.message || "删除失败");
      }
    },
  });
}

function handleEdit(id: string) {
  const p = projectStore.projects.find((x) => x.id === id);
  if (!p) return;
  editingId.value = id;
  editForm.value = {
    title: p.title,
    genre: p.genre,
    summary: p.summary,
    target_words: p.target_words,
  };
  showEdit.value = true;
}

async function handleUpdate() {
  if (!editingId.value) return;
  if (!editForm.value.title.trim()) {
    message.warning("请输入项目名称");
    return;
  }
  editing.value = true;
  try {
    await projectStore.updateProject(editingId.value, {
      title: editForm.value.title.trim(),
      genre: editForm.value.genre,
      summary: editForm.value.summary,
      target_words: editForm.value.target_words,
    });
    message.success("项目已更新");
    showEdit.value = false;
  } catch (e: any) {
    message.error(e?.message || "更新失败");
  } finally {
    editing.value = false;
  }
}

const importing = ref(false);
async function handleImport() {
  try {
    const selected = await open({
      multiple: false,
      filters: [{ name: "JSON 项目文件", extensions: ["json"] }],
      title: "选择要导入的项目文件",
    });
    if (!selected) return;

    importing.value = true;
    const newProjectId = await importProject(selected as string);
    message.success("项目导入成功");
    await projectStore.fetchProjects();
    router.push({ name: "editor", params: { id: newProjectId } });
  } catch (e: any) {
    message.error(e?.message || "导入失败");
  } finally {
    importing.value = false;
  }
}

onMounted(() => {
  projectStore.fetchProjects();
});
</script>

<template>
  <div class="min-h-screen bg-gray-50 dark:bg-gray-900 text-gray-900 dark:text-gray-100 transition-colors">
    <!-- 顶栏 -->
    <header
      class="sticky top-0 z-10 border-b border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 px-6 py-4"
    >
      <div class="max-w-6xl mx-auto flex items-center justify-between">
        <div class="flex items-center gap-3 text-linden-primary">
          <div class="flex flex-col leading-tight">
            <h1 class="text-xl font-bold text-linden-primary">菩提思</h1>
            <span class="text-xs font-normal text-gray-400 dark:text-gray-500">助你文思泉涌，妙笔生花</span>
          </div>
        </div>
        <div class="flex items-center gap-3">
          <!-- 项目操作 -->
          <NButton @click="handleImport" :loading="importing">
            <template #icon>
              <span class="i-carbon-upload" />
            </template>
            导入项目
          </NButton>
          <NButton type="primary" @click="showCreate = true">
            <template #icon>
              <span class="i-carbon-add" />
            </template>
            新建项目
          </NButton>
          <!-- 分隔线 -->
          <div class="w-px h-6 bg-gray-200 dark:bg-gray-600" />
          <!-- 应用设置：工具组 -->
          <div class="flex items-center gap-0.5 p-0.5 rounded-lg bg-gray-100 dark:bg-gray-700/40">
            <NButton quaternary size="small" @click="showAiSettings = true">
              <template #icon>
                <span class="i-carbon-ai" />
              </template>
              AI 设置
            </NButton>
            <NButton quaternary size="small" @click="toggle">
              <template #icon>
                <span v-if="isDark" class="i-carbon-moon" />
                <span v-else class="i-carbon-sun" />
              </template>
              {{ isDark ? "暗色" : "亮色" }}
            </NButton>
          </div>
        </div>
      </div>
    </header>

    <!-- 项目列表 -->
    <main class="max-w-6xl mx-auto px-6 py-6">
      <NSpin :show="projectStore.loading">
        <NEmpty
          v-if="!projectStore.loading && projectStore.projects.length === 0"
          description="还没有项目，点击「新建项目」开始创作吧"
          class="py-20"
        >
          <template #extra>
            <NButton type="primary" @click="showCreate = true">
              新建项目
            </NButton>
          </template>
        </NEmpty>

        <NGrid
          v-else
          :cols="3"
          :x-gap="16"
          :y-gap="16"
          responsive="screen"
          item-responsive
        >
          <NGi
            v-for="project in projectStore.projects"
            :key="project.id"
            span="0:3 640:2 1024:1"
          >
            <ProjectCard
              :project="project"
              @open="handleOpen"
              @delete="handleDelete"
              @edit="handleEdit"
            />
          </NGi>
        </NGrid>
      </NSpin>
    </main>

    <!-- 新建项目弹窗 -->
    <NModal
      v-model:show="showCreate"
      preset="dialog"
      title="新建项目"
      positive-text="创建"
      negative-text="取消"
      :loading="creating"
      @positive-click="handleCreate"
    >
      <NForm label-placement="left" label-width="80">
        <NFormItem label="项目名称">
          <NInput
            v-model:value="createForm.title"
            placeholder="输入小说标题"
            maxlength="100"
            show-count
          />
        </NFormItem>
        <NFormItem label="题材类型">
          <NSelect
            v-model:value="createForm.genre"
            :options="genreOptions"
            placeholder="选择题材（可选）"
            clearable
          />
        </NFormItem>
        <NFormItem label="目标字数">
          <NInputNumber
            v-model:value="createForm.target_words"
            placeholder="如 300000"
            :step="10000"
            :min="0"
            clearable
          >
            <template #suffix>字</template>
          </NInputNumber>
        </NFormItem>
        <NFormItem label="简介">
          <NInput
            v-model:value="createForm.summary"
            type="textarea"
            placeholder="简述故事梗概（可选）"
            :rows="3"
            maxlength="500"
          />
        </NFormItem>
      </NForm>
    </NModal>

    <!-- 编辑项目弹窗 -->
    <NModal
      v-model:show="showEdit"
      preset="dialog"
      title="编辑项目"
      positive-text="保存"
      negative-text="取消"
      :loading="editing"
      @positive-click="handleUpdate"
    >
      <NForm label-placement="left" label-width="80">
        <NFormItem label="项目名称">
          <NInput
            v-model:value="editForm.title"
            placeholder="输入小说标题"
            maxlength="100"
            show-count
          />
        </NFormItem>
        <NFormItem label="题材类型">
          <NSelect
            v-model:value="editForm.genre"
            :options="genreOptions"
            placeholder="选择题材（可选）"
            clearable
          />
        </NFormItem>
        <NFormItem label="目标字数">
          <NInputNumber
            v-model:value="editForm.target_words"
            placeholder="如 300000"
            :step="10000"
            :min="0"
            clearable
          >
            <template #suffix>字</template>
          </NInputNumber>
        </NFormItem>
        <NFormItem label="简介">
          <NInput
            v-model:value="editForm.summary"
            type="textarea"
            placeholder="简述故事梗概（可选）"
            :rows="3"
            maxlength="500"
          />
        </NFormItem>
      </NForm>
    </NModal>

    <!-- AI 设置对话框 -->
    <AiSettingsDialog v-model:show="showAiSettings" />
  </div>
</template>
