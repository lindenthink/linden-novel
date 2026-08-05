<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import {
  NButton,
  NSpace,
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
import ProjectCard from "../components/common/ProjectCard.vue";

const router = useRouter();
const projectStore = useProjectStore();
const message = useMessage();
const dialog = useDialog();

// ---- 新建项目弹窗 ----
const showCreate = ref(false);
const createForm = ref({
  title: "",
  genre: null as string | null,
  summary: null as string | null,
  target_words: null as number | null,
});
const creating = ref(false);

const genreOptions = [
  { label: "奇幻", value: "奇幻" },
  { label: "科幻", value: "科幻" },
  { label: "言情", value: "言情" },
  { label: "悬疑", value: "悬疑" },
  { label: "武侠", value: "武侠" },
  { label: "历史", value: "历史" },
  { label: "现实", value: "现实" },
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

function handleEdit(_id: string) {
  // TODO: P5+ 实现项目编辑弹窗，暂时用 inline rename
  message.info("项目编辑功能后续实现");
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
        <div class="flex items-center gap-3">
          <span class="i-carbon-book text-2xl text-linden-primary" />
          <h1 class="text-xl font-bold">Linden Novel</h1>
        </div>
        <NSpace>
          <NButton type="primary" @click="showCreate = true">
            <template #icon>
              <span class="i-carbon-add" />
            </template>
            新建项目
          </NButton>
        </NSpace>
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
  </div>
</template>
