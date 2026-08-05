<script setup lang="ts">
import { ref, onMounted } from "vue";
import { NModal, NForm, NFormItem, NInput, useMessage } from "naive-ui";
import { useElementStore } from "../../stores/element";
import { useProjectStore } from "../../stores/project";
import ElementList from "./ElementList.vue";
import type { WorldviewEntry } from "../../types";

const elementStore = useElementStore();
const projectStore = useProjectStore();
const message = useMessage();

const showEdit = ref(false);
const editingItem = ref<WorldviewEntry | null>(null);
const form = ref({
  name: "",
  category: "",
  description: "",
});

onMounted(() => {
  if (projectStore.currentProject) {
    elementStore.fetchWorldview(projectStore.currentProject.id);
  }
});

function handleCreate() {
  editingItem.value = null;
  form.value = { name: "", category: "", description: "" };
  showEdit.value = true;
}

function handleEdit(item: { id: string; name: string; description?: string | null }) {
  const full = elementStore.worldview.find((w) => w.id === item.id);
  if (!full) return;
  editingItem.value = full;
  form.value = {
    name: full.name,
    category: full.category || "",
    description: full.description || "",
  };
  showEdit.value = true;
}

async function handleSave() {
  if (!form.value.name.trim()) {
    message.warning("请输入条目名称");
    return;
  }

  try {
    if (editingItem.value) {
      await elementStore.updateWorldview(editingItem.value.id, {
        name: form.value.name,
        category: form.value.category || null,
        description: form.value.description || null,
      });
      message.success("条目已更新");
    } else {
      if (!projectStore.currentProject) return;
      await elementStore.createWorldview({
        project_id: projectStore.currentProject.id,
        name: form.value.name,
        category: form.value.category || null,
        description: form.value.description || null,
      });
      message.success("条目已创建");
    }
    showEdit.value = false;
  } catch (e: any) {
    message.error(e?.message || "操作失败");
  }
}

async function handleDelete(id: string) {
  try {
    await elementStore.deleteWorldview(id);
  } catch (e: any) {
    message.error(e?.message || "删除失败");
  }
}
</script>

<template>
  <div class="worldview-panel flex flex-col h-full">
    <ElementList
      :items="elementStore.worldview"
      @create="handleCreate"
      @edit="handleEdit"
      @delete="handleDelete"
    />

    <NModal
      v-model:show="showEdit"
      preset="dialog"
      :title="editingItem ? '编辑条目' : '新建条目'"
      positive-text="保存"
      negative-text="取消"
      @positive-click="handleSave"
    >
      <NForm label-placement="left" label-width="60">
        <NFormItem label="名称">
          <NInput v-model:value="form.name" placeholder="条目名称" maxlength="50" show-count />
        </NFormItem>
        <NFormItem label="分类">
          <NInput v-model:value="form.category" placeholder="如：地理、势力、规则" maxlength="30" />
        </NFormItem>
        <NFormItem label="描述">
          <NInput
            v-model:value="form.description"
            type="textarea"
            placeholder="条目描述"
            :rows="4"
            maxlength="500"
          />
        </NFormItem>
      </NForm>
    </NModal>
  </div>
</template>
