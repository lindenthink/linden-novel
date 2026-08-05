<script setup lang="ts">
import { ref, onMounted } from "vue";
import { NModal, NForm, NFormItem, NInput, NSelect, useMessage } from "naive-ui";
import { useElementStore } from "../../stores/element";
import { useProjectStore } from "../../stores/project";
import ElementList from "./ElementList.vue";
import type { Storyline } from "../../types";

const elementStore = useElementStore();
const projectStore = useProjectStore();
const message = useMessage();

const showEdit = ref(false);
const editingItem = ref<Storyline | null>(null);
const form = ref({
  name: "",
  description: "",
  status: "active" as "active" | "completed" | "abandoned",
});

const statusOptions = [
  { label: "进行中", value: "active" },
  { label: "已完成", value: "completed" },
  { label: "已放弃", value: "abandoned" },
];

onMounted(() => {
  if (projectStore.currentProject) {
    elementStore.fetchStorylines(projectStore.currentProject.id);
  }
});

function handleCreate() {
  editingItem.value = null;
  form.value = { name: "", description: "", status: "active" };
  showEdit.value = true;
}

function handleEdit(item: { id: string; name: string; description?: string | null }) {
  const full = elementStore.storylines.find((s) => s.id === item.id);
  if (!full) return;
  editingItem.value = full;
  form.value = {
    name: full.name,
    description: full.description || "",
    status: full.status,
  };
  showEdit.value = true;
}

async function handleSave() {
  if (!form.value.name.trim()) {
    message.warning("请输入故事线名称");
    return;
  }

  try {
    if (editingItem.value) {
      await elementStore.updateStoryline(editingItem.value.id, {
        name: form.value.name,
        description: form.value.description || null,
        status: form.value.status,
      });
      message.success("故事线已更新");
    } else {
      if (!projectStore.currentProject) return;
      await elementStore.createStoryline({
        project_id: projectStore.currentProject.id,
        name: form.value.name,
        description: form.value.description || null,
      });
      message.success("故事线已创建");
    }
    showEdit.value = false;
  } catch (e: any) {
    message.error(e?.message || "操作失败");
  }
}

async function handleDelete(id: string) {
  try {
    await elementStore.deleteStoryline(id);
  } catch (e: any) {
    message.error(e?.message || "删除失败");
  }
}
</script>

<template>
  <div class="storyline-panel flex flex-col h-full">
    <ElementList
      :items="elementStore.storylines"
      @create="handleCreate"
      @edit="handleEdit"
      @delete="handleDelete"
    />

    <NModal
      v-model:show="showEdit"
      preset="dialog"
      :title="editingItem ? '编辑故事线' : '新建故事线'"
      positive-text="保存"
      negative-text="取消"
      @positive-click="handleSave"
    >
      <NForm label-placement="left" label-width="60">
        <NFormItem label="名称">
          <NInput v-model:value="form.name" placeholder="故事线名称" maxlength="50" show-count />
        </NFormItem>
        <NFormItem label="状态">
          <NSelect v-model:value="form.status" :options="statusOptions" />
        </NFormItem>
        <NFormItem label="描述">
          <NInput
            v-model:value="form.description"
            type="textarea"
            placeholder="故事线描述"
            :rows="4"
            maxlength="500"
          />
        </NFormItem>
      </NForm>
    </NModal>
  </div>
</template>
