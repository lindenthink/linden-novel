<script setup lang="ts">
import { ref, onMounted } from "vue";
import { NModal, NForm, NFormItem, NInput, useMessage } from "naive-ui";
import { useElementStore } from "../../stores/element";
import { useProjectStore } from "../../stores/project";
import ElementList from "./ElementList.vue";
import type { Character } from "../../types";

const elementStore = useElementStore();
const projectStore = useProjectStore();
const message = useMessage();

const showEdit = ref(false);
const editingItem = ref<Character | null>(null);
const form = ref({
  name: "",
  role: "",
  description: "",
});

onMounted(() => {
  if (projectStore.currentProject) {
    elementStore.fetchCharacters(projectStore.currentProject.id);
  }
});

function handleCreate() {
  editingItem.value = null;
  form.value = { name: "", role: "", description: "" };
  showEdit.value = true;
}

function handleEdit(item: { id: string; name: string; description?: string | null }) {
  const full = elementStore.characters.find((c) => c.id === item.id);
  if (!full) return;
  editingItem.value = full;
  form.value = {
    name: full.name,
    role: full.role || "",
    description: full.description || "",
  };
  showEdit.value = true;
}

async function handleSave() {
  if (!form.value.name.trim()) {
    message.warning("请输入人物姓名");
    return;
  }

  try {
    if (editingItem.value) {
      await elementStore.updateCharacter(editingItem.value.id, {
        name: form.value.name,
        role: form.value.role || null,
        description: form.value.description || null,
      });
      message.success("人物已更新");
    } else {
      if (!projectStore.currentProject) return;
      await elementStore.createCharacter({
        project_id: projectStore.currentProject.id,
        name: form.value.name,
        role: form.value.role || null,
        description: form.value.description || null,
      });
      message.success("人物已创建");
    }
    showEdit.value = false;
  } catch (e: any) {
    message.error(e?.message || "操作失败");
  }
}

async function handleDelete(id: string) {
  try {
    await elementStore.deleteCharacter(id);
  } catch (e: any) {
    message.error(e?.message || "删除失败");
  }
}
</script>

<template>
  <div class="character-panel flex flex-col h-full">
    <ElementList
      :items="elementStore.characters"
      @create="handleCreate"
      @edit="handleEdit"
      @delete="handleDelete"
    />

    <!-- 编辑弹窗 -->
    <NModal
      v-model:show="showEdit"
      preset="dialog"
      :title="editingItem ? '编辑人物' : '新建人物'"
      positive-text="保存"
      negative-text="取消"
      @positive-click="handleSave"
    >
      <NForm label-placement="left" label-width="60">
        <NFormItem label="姓名">
          <NInput v-model:value="form.name" placeholder="人物姓名" maxlength="50" show-count />
        </NFormItem>
        <NFormItem label="角色">
          <NInput v-model:value="form.role" placeholder="如：主角、配角、反派" maxlength="30" />
        </NFormItem>
        <NFormItem label="描述">
          <NInput
            v-model:value="form.description"
            type="textarea"
            placeholder="性格、外貌、背景等"
            :rows="4"
            maxlength="500"
          />
        </NFormItem>
      </NForm>
    </NModal>
  </div>
</template>
