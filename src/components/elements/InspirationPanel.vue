<script setup lang="ts">
import { ref, onMounted } from "vue";
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
import type {
  Inspiration,
  CreateInspiration,
  UpdateInspiration,
  InspirationStatus,
} from "../../types";
import { formatLocalTime } from "../../utils/time";

const elementStore = useElementStore();
const projectStore = useProjectStore();
const message = useMessage();

const showEdit = ref(false);
const editingItem = ref<Inspiration | null>(null);

const form = ref({
  content: "",
  tag: null as string | null,
  status: "new" as InspirationStatus,
});

const statusTagType = (
  status: InspirationStatus,
): "success" | "info" | "default" => {
  return status === "adopted" ? "success" : status === "new" ? "info" : "default";
};

const statusLabel = (status: InspirationStatus): string => {
  return status === "adopted" ? "已采用" : status === "new" ? "待处理" : "已搁置";
};

const statusOptions = [
  { label: "待处理", value: "new" },
  { label: "已采用", value: "adopted" },
  { label: "已搁置", value: "shelved" },
];

// 常用标签快捷选项（也允许自由输入）
const tagOptions = [
  "情节",
  "人物",
  "设定",
  "文风",
  "其他",
].map((t) => ({ label: t, value: t }));

onMounted(() => {
  if (projectStore.currentProject) {
    elementStore.fetchInspirations(projectStore.currentProject.id);
  }
});

function handleCreate() {
  editingItem.value = null;
  form.value = { content: "", tag: null, status: "new" };
  showEdit.value = true;
}

function handleEdit(item: Inspiration) {
  editingItem.value = item;
  form.value = {
    content: item.content,
    tag: item.tag,
    status: item.status,
  };
  showEdit.value = true;
}

async function handleSave() {
  if (!form.value.content.trim()) {
    message.warning("请输入灵感内容");
    return;
  }

  try {
    if (editingItem.value) {
      const input: UpdateInspiration = {
        content: form.value.content,
        tag: form.value.tag,
        status: form.value.status,
      };
      await elementStore.updateInspiration(editingItem.value.id, input);
      message.success("灵感已更新");
    } else {
      if (!projectStore.currentProject) return;
      const input: CreateInspiration = {
        project_id: projectStore.currentProject.id,
        content: form.value.content,
        tag: form.value.tag,
        status: form.value.status,
      };
      await elementStore.createInspiration(input);
      message.success("灵感已记录");
    }
    showEdit.value = false;
  } catch (e: any) {
    message.error(e?.message || "操作失败");
  }
}

async function handleDelete(id: string) {
  try {
    await elementStore.deleteInspiration(id);
    message.success("已删除");
  } catch (e: any) {
    message.error(e?.message || "删除失败");
  }
}
</script>

<template>
  <div class="inspiration-panel flex flex-col h-full">
    <!-- 顶部操作栏 -->
    <div class="flex items-center justify-between px-3 py-2 border-b border-gray-200 dark:border-gray-700">
      <span class="text-sm font-medium">灵感记录</span>
      <NButton size="small" type="primary" @click="handleCreate">
        <template #icon>
          <span class="i-carbon-add" />
        </template>
        新建
      </NButton>
    </div>

    <!-- 列表（按创建时间倒序） -->
    <div class="flex-1 overflow-auto">
      <NEmpty
        v-if="elementStore.inspirations.length === 0"
        description="暂无灵感，随时记录闪现的想法"
        class="py-10"
        size="small"
      />
      <NList v-else hoverable clickable>
        <NListItem
          v-for="item in elementStore.inspirations"
          :key="item.id"
          @click="handleEdit(item)"
        >
          <div class="flex flex-col w-full min-w-0">
            <div class="flex items-center gap-2 flex-wrap">
              <NTag v-if="item.tag" size="small" round :bordered="false" class="flex-shrink-0">
                {{ item.tag }}
              </NTag>
              <NTag size="small" round :type="statusTagType(item.status)" class="flex-shrink-0">
                {{ statusLabel(item.status) }}
              </NTag>
              <span class="text-xs text-gray-400 dark:text-gray-500 ml-auto flex-shrink-0">
                {{ formatLocalTime(item.created_at) }}
              </span>
            </div>
            <div class="text-sm mt-1.5 line-clamp-3 break-words whitespace-pre-wrap">
              {{ item.content }}
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
              确定删除这条灵感吗？
            </NPopconfirm>
          </template>
        </NListItem>
      </NList>
    </div>

    <!-- 新建/编辑弹窗 -->
    <NModal
      v-model:show="showEdit"
      preset="dialog"
      :title="editingItem ? '编辑灵感' : '记录灵感'"
      positive-text="保存"
      negative-text="取消"
      @positive-click="handleSave"
      style="width: 500px; max-width: 90vw;"
    >
      <NForm label-placement="left" label-width="60">
        <NFormItem label="内容">
          <NInput
            v-model:value="form.content"
            type="textarea"
            placeholder="记录闪现的灵感..."
            :rows="5"
            maxlength="2000"
            show-count
          />
        </NFormItem>
        <div class="flex gap-4">
          <NFormItem label="标签" class="flex-1">
            <NSelect
              v-model:value="form.tag"
              :options="tagOptions"
              tag
              filterable
              clearable
              placeholder="选择或输入标签"
            />
          </NFormItem>
          <NFormItem label="状态" class="flex-1">
            <NSelect v-model:value="form.status" :options="statusOptions" />
          </NFormItem>
        </div>
      </NForm>
    </NModal>
  </div>
</template>
