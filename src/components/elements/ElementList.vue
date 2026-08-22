<script setup lang="ts">
import { ref, computed } from "vue";
import { NButton, NInput, NList, NListItem, NEmpty, NPopconfirm, useMessage } from "naive-ui";

const props = defineProps<{
  items: Array<{ id: string; name: string; description?: string | null }>;
  loading?: boolean;
}>();

const emit = defineEmits<{
  create: [];
  edit: [item: { id: string; name: string; description?: string | null }];
  delete: [id: string];
}>();

const message = useMessage();
const searchQuery = ref("");

const filteredItems = computed(() => {
  if (!searchQuery.value.trim()) return props.items;
  const query = searchQuery.value.toLowerCase();
  return props.items.filter(
    (item) =>
      item.name.toLowerCase().includes(query) ||
      item.description?.toLowerCase().includes(query),
  );
});

function handleEdit(item: { id: string; name: string; description?: string | null }) {
  emit("edit", item);
}

function handleDelete(id: string) {
  emit("delete", id);
  message.success("已删除");
}
</script>

<template>
  <div class="element-list flex flex-col h-full">
    <!-- 搜索栏 -->
    <div class="flex items-center gap-2 px-3 py-2 border-b border-gray-200 dark:border-gray-700">
      <NInput
        v-model:value="searchQuery"
        placeholder="搜索..."
        size="small"
        clearable
      />
      <NButton size="small" type="primary" @click="emit('create')">
        <template #icon>
          <span class="i-carbon-add" />
        </template>
      </NButton>
    </div>

    <!-- 列表 -->
    <div class="flex-1 overflow-auto">
      <NEmpty v-if="filteredItems.length === 0" description="暂无数据" class="py-10" size="small" />
      <NList v-else hoverable clickable>
        <NListItem
          v-for="item in filteredItems"
          :key="item.id"
          @click="handleEdit(item)"
        >
          <div class="flex items-center justify-between w-full">
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-2">
                <span class="font-medium truncate">{{ item.name }}</span>
                <slot name="item-meta" :item="item" />
              </div>
              <div v-if="item.description" class="text-xs text-gray-500 dark:text-gray-400 mt-1 line-clamp-2 break-words">
                {{ item.description }}
              </div>
            </div>
            <NPopconfirm @positive-click="handleDelete(item.id)">
              <template #trigger>
                <NButton quaternary size="tiny" class="ml-2" @click.stop>
                  <template #icon>
                    <span class="i-carbon-trash-can" />
                  </template>
                </NButton>
              </template>
              确定删除吗？
            </NPopconfirm>
          </div>
        </NListItem>
      </NList>
    </div>
  </div>
</template>
