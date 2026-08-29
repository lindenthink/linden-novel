<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import {
  NModal,
  NForm,
  NFormItem,
  NInput,
  NSelect,
  NTag,
  NEmpty,
  NList,
  NListItem,
  NButton,
  NPopconfirm,
  useMessage,
} from "naive-ui";
import { useElementStore } from "../../stores/element";
import { useProjectStore } from "../../stores/project";
import { useChapterStore } from "../../stores/chapter";
import type {
  Foreshadow,
  CreateForeshadow,
  UpdateForeshadow,
  ForeshadowImportance,
  ForeshadowStatus,
} from "../../types";
import { formatLocalTime } from "../../utils/time";

const elementStore = useElementStore();
const projectStore = useProjectStore();
const chapterStore = useChapterStore();
const message = useMessage();

const showEdit = ref(false);
const editingItem = ref<Foreshadow | null>(null);

const form = ref({
  title: "",
  description: "",
  importance: "normal" as ForeshadowImportance,
  status: "planted" as ForeshadowStatus,
  plant_chapter_id: null as string | null,
  resolve_chapter_id: null as string | null,
  plant_note: "",
  resolve_note: "",
});

const importanceTagType = (
  importance: ForeshadowImportance,
): "error" | "warning" | "info" | "default" => {
  return importance === "major"
    ? "error"
    : importance === "normal"
      ? "warning"
      : "info";
};

const importanceLabel = (importance: ForeshadowImportance): string => {
  return importance === "major" ? "重要" : importance === "normal" ? "普通" : "次要";
};

const statusTagType = (
  status: ForeshadowStatus,
): "success" | "info" | "warning" | "error" | "default" => {
  return status === "resolved"
    ? "success"
    : status === "planted"
      ? "warning"
      : status === "pending"
        ? "info"
        : "default";
};

const statusLabel = (status: ForeshadowStatus): string => {
  return status === "resolved"
    ? "已回收"
    : status === "planted"
      ? "已埋下"
      : status === "pending"
        ? "待埋"
        : "已废弃";
};

const importanceOptions = [
  { label: "重要", value: "major" },
  { label: "普通", value: "normal" },
  { label: "次要", value: "minor" },
];

const statusOptions = [
  { label: "待埋", value: "pending" },
  { label: "已埋下", value: "planted" },
  { label: "已回收", value: "resolved" },
  { label: "已废弃", value: "abandoned" },
];

// 章节选项：用于选择埋点/回收章节
const chapterOptions = computed(() =>
  chapterStore.chapters.map((c) => ({
    label: c.title,
    value: c.id,
  })),
);

// 章节标题映射，用于列表展示
const chapterTitleMap = computed(() => {
  const map = new Map<string, string>();
  chapterStore.chapters.forEach((c) => map.set(c.id, c.title));
  return map;
});

onMounted(() => {
  if (projectStore.currentProject) {
    elementStore.fetchForeshadows(projectStore.currentProject.id);
  }
});

function handleCreate() {
  editingItem.value = null;
  form.value = {
    title: "",
    description: "",
    importance: "normal",
    status: "planted",
    plant_chapter_id: chapterStore.activeChapterId,
    resolve_chapter_id: null,
    plant_note: "",
    resolve_note: "",
  };
  showEdit.value = true;
}

function handleEdit(item: Foreshadow) {
  editingItem.value = item;
  form.value = {
    title: item.title,
    description: item.description || "",
    importance: item.importance,
    status: item.status,
    plant_chapter_id: item.plant_chapter_id,
    resolve_chapter_id: item.resolve_chapter_id,
    plant_note: item.plant_note || "",
    resolve_note: item.resolve_note || "",
  };
  showEdit.value = true;
}

async function handleSave() {
  if (!form.value.title.trim()) {
    message.warning("请输入伏笔标题");
    return;
  }

  try {
    if (editingItem.value) {
      const input: UpdateForeshadow = {
        title: form.value.title,
        description: form.value.description || null,
        importance: form.value.importance,
        status: form.value.status,
        plant_chapter_id: form.value.plant_chapter_id,
        resolve_chapter_id: form.value.resolve_chapter_id,
        plant_note: form.value.plant_note || null,
        resolve_note: form.value.resolve_note || null,
      };
      await elementStore.updateForeshadow(editingItem.value.id, input);
      message.success("伏笔已更新");
    } else {
      if (!projectStore.currentProject) return;
      const input: CreateForeshadow = {
        project_id: projectStore.currentProject.id,
        title: form.value.title,
        description: form.value.description || null,
        importance: form.value.importance,
        status: form.value.status,
        plant_chapter_id: form.value.plant_chapter_id,
        resolve_chapter_id: form.value.resolve_chapter_id,
        plant_note: form.value.plant_note || null,
        resolve_note: form.value.resolve_note || null,
      };
      await elementStore.createForeshadow(input);
      message.success("伏笔已创建");
    }
    showEdit.value = false;
  } catch (e: any) {
    message.error(e?.message || "操作失败");
  }
}

async function handleDelete(id: string) {
  try {
    await elementStore.deleteForeshadow(id);
    message.success("已删除");
  } catch (e: any) {
    message.error(e?.message || "删除失败");
  }
}
</script>

<template>
  <div class="foreshadow-panel flex flex-col h-full">
    <!-- 顶部操作栏 -->
    <div class="flex items-center justify-between px-3 py-2 border-b border-gray-200 dark:border-gray-700">
      <span class="text-sm font-medium">伏笔管理</span>
      <NButton size="small" type="primary" @click="handleCreate">
        <template #icon>
          <span class="i-carbon-add" />
        </template>
        新建
      </NButton>
    </div>

    <!-- 列表 -->
    <div class="flex-1 overflow-auto">
      <NEmpty
        v-if="elementStore.foreshadows.length === 0"
        description="暂无伏笔"
        class="py-10"
        size="small"
      />
      <NList v-else hoverable clickable>
        <NListItem
          v-for="item in elementStore.foreshadows"
          :key="item.id"
          @click="handleEdit(item)"
        >
          <div class="flex flex-col w-full min-w-0">
            <div class="flex items-center gap-2 flex-wrap">
              <span class="font-medium truncate">{{ item.title }}</span>
              <NTag size="small" round :type="statusTagType(item.status)" class="flex-shrink-0">
                {{ statusLabel(item.status) }}
              </NTag>
              <NTag
                size="small"
                round
                :type="importanceTagType(item.importance)"
                class="flex-shrink-0"
              >
                {{ importanceLabel(item.importance) }}
              </NTag>
            </div>
            <div
              v-if="item.description"
              class="text-xs text-gray-500 dark:text-gray-400 mt-1 line-clamp-2 break-words"
            >
              {{ item.description }}
            </div>
            <div class="flex items-center gap-3 mt-1 text-xs text-gray-400 dark:text-gray-500">
              <span v-if="item.plant_chapter_id">
                埋：{{ chapterTitleMap.get(item.plant_chapter_id) || "未知章节" }}
              </span>
              <span v-if="item.resolve_chapter_id">
                收：{{ chapterTitleMap.get(item.resolve_chapter_id) || "未知章节" }}
              </span>
              <span>{{ formatLocalTime(item.created_at) }}</span>
            </div>
          </div>
          <template #suffix>
            <NPopconfirm @positive-click="handleDelete(item.id)">
              <template #trigger>
                <NButton quaternary size="tiny" @click.stop>
                  <template #icon>
                    <span class="i-carbon-trash-can" />
                  </template>
                </NButton>
              </template>
              确定删除吗？
            </NPopconfirm>
          </template>
        </NListItem>
      </NList>
    </div>

    <!-- 编辑/新建弹窗 -->
    <NModal
      v-model:show="showEdit"
      preset="dialog"
      :title="editingItem ? '编辑伏笔' : '新建伏笔'"
      positive-text="保存"
      negative-text="取消"
      @positive-click="handleSave"
      style="width: 600px; max-width: 90vw;"
    >
      <NForm label-placement="left" label-width="80">
        <NFormItem label="标题">
          <NInput v-model:value="form.title" placeholder="伏笔标题" maxlength="50" show-count />
        </NFormItem>
        <NFormItem label="描述">
          <NInput
            v-model:value="form.description"
            type="textarea"
            placeholder="伏笔内容描述"
            :rows="3"
            maxlength="500"
          />
        </NFormItem>
        <div class="flex gap-4">
          <NFormItem label="重要性" class="flex-1">
            <NSelect v-model:value="form.importance" :options="importanceOptions" />
          </NFormItem>
          <NFormItem label="状态" class="flex-1">
            <NSelect v-model:value="form.status" :options="statusOptions" />
          </NFormItem>
        </div>
        <NFormItem label="埋点章节">
          <NSelect
            v-model:value="form.plant_chapter_id"
            :options="chapterOptions"
            clearable
            placeholder="选择埋下伏笔的章节"
          />
        </NFormItem>
        <NFormItem label="回收章节">
          <NSelect
            v-model:value="form.resolve_chapter_id"
            :options="chapterOptions"
            clearable
            placeholder="选择回收伏笔的章节（未回收留空）"
          />
        </NFormItem>
        <NFormItem label="埋点说明">
          <NInput
            v-model:value="form.plant_note"
            type="textarea"
            placeholder="埋点位置/上下文说明"
            :rows="2"
            maxlength="300"
          />
        </NFormItem>
        <NFormItem label="回收说明">
          <NInput
            v-model:value="form.resolve_note"
            type="textarea"
            placeholder="回收方式说明"
            :rows="2"
            maxlength="300"
          />
        </NFormItem>
      </NForm>
    </NModal>
  </div>
</template>
