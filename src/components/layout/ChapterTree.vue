<script setup lang="ts">
import { computed, h, ref } from "vue";
import {
  NTree,
  NDropdown,
  NEmpty,
  NButton,
  useMessage,
  useDialog,
  type TreeOption,
} from "naive-ui";
import { useProjectStore } from "../../stores/project";
import { useChapterStore } from "../../stores/chapter";
import type { Chapter } from "../../types";

// ---- 扩展 TreeOption 携带自定义数据 ----
interface ChapterTreeNode extends TreeOption {
  isVolume?: boolean;
  volumeId?: string;
  chapterId?: string;
  chapter?: Chapter;
}

const projectStore = useProjectStore();
const chapterStore = useChapterStore();
const message = useMessage();
const dialog = useDialog();

// ---- 右键菜单 ----
const contextmenu = ref(false);
const contextNode = ref<ChapterTreeNode | null>(null);
const contextX = ref(0);
const contextY = ref(0);

// ---- 将 volumes + chapters 转为 TreeOption ----
const treeData = computed<ChapterTreeNode[]>(() => {
  const volumes = projectStore.volumes;
  const chapters = chapterStore.chapters;

  return volumes.map((vol): ChapterTreeNode => ({
    key: `vol-${vol.id}`,
    label: vol.title,
    isVolume: true,
    volumeId: vol.id,
    prefix: () => h("span", { class: "i-carbon-folder mr-1 opacity-60" }),
    children: chapters
      .filter((ch) => ch.volume_id === vol.id)
      .sort((a, b) => a.order_index - b.order_index)
      .map((ch): ChapterTreeNode => ({
        key: `ch-${ch.id}`,
        label: ch.title,
        isVolume: false,
        chapterId: ch.id,
        chapter: ch,
        prefix: () =>
          h("span", {
            class: ch.status === "final"
              ? "i-carbon-checkmark-filled mr-1 text-green-500"
              : ch.status === "writing"
                ? "i-carbon-edit mr-1 text-blue-500"
                : "i-carbon-document mr-1 opacity-40",
          }),
      })),
  }));
});

// ---- 选中 ----
const selectedKeys = computed(() =>
  chapterStore.activeChapterId ? [`ch-${chapterStore.activeChapterId}`] : [],
);

function handleSelect(keys: string[]) {
  if (keys.length === 0) return;
  const key = keys[0];
  if (key.startsWith("ch-")) {
    const chapterId = key.slice(3);
    chapterStore.setActiveChapter(chapterId);
  }
}

// ---- 右键菜单 ----
function getContextMenu(node: ChapterTreeNode) {
  if (node.isVolume) {
    return [
      { label: "新建章节", key: "add-chapter" },
      { label: "重命名卷", key: "rename-vol" },
      { label: "删除卷", key: "delete-vol" },
    ];
  }
  return [
    { label: "重命名章节", key: "rename-ch" },
    { label: "标记为写作中", key: "status-writing", disabled: node.chapter?.status === "writing" },
    { label: "标记为定稿", key: "status-final", disabled: node.chapter?.status === "final" },
    { label: "标记为草稿", key: "status-draft", disabled: node.chapter?.status === "draft" },
    { type: "divider" as const, key: "d1" },
    { label: "删除章节", key: "delete-ch" },
  ];
}

function handleRightClick({ event, node }: { event: MouseEvent; node: ChapterTreeNode }) {
  event.preventDefault();
  contextNode.value = node;
  contextX.value = event.clientX;
  contextY.value = event.clientY;
  contextmenu.value = true;
}

async function handleContextSelect(key: string) {
  const node = contextNode.value;
  if (!node) return;
  contextmenu.value = false;

  try {
    // ---- 卷操作 ----
    if (key === "add-chapter" && node.volumeId) {
      const title = `第 ${chapterStore.chapters.filter((c) => c.volume_id === node.volumeId).length + 1} 章`;
      await chapterStore.createChapter(
        node.volumeId,
        projectStore.currentProject!.id,
        title,
      );
      message.success("章节已创建");
    }

    if (key === "rename-vol" && node.volumeId) {
      const newTitle = prompt("输入新的卷名：", node.label as string);
      if (newTitle?.trim()) {
        await projectStore.updateVolume(node.volumeId, { title: newTitle.trim() });
        message.success("已重命名");
      }
    }

    if (key === "delete-vol" && node.volumeId) {
      dialog.warning({
        title: "确认删除",
        content: `确定删除卷「${node.label}」及其所有章节吗？此操作不可撤销。`,
        positiveText: "删除",
        negativeText: "取消",
        onPositiveClick: async () => {
          // 先删除该卷下的所有章节
          const volChapters = chapterStore.chapters.filter(
            (c) => c.volume_id === node.volumeId,
          );
          for (const ch of volChapters) {
            await chapterStore.deleteChapter(ch.id);
          }
          await projectStore.deleteVolume(node.volumeId!);
          message.success("已删除");
        },
      });
    }

    // ---- 章节操作 ----
    if (key === "rename-ch" && node.chapterId) {
      const newTitle = prompt("输入新的章节名：", node.label as string);
      if (newTitle?.trim()) {
        await chapterStore.updateChapterMeta(node.chapterId, { title: newTitle.trim() });
        message.success("已重命名");
      }
    }

    if (key.startsWith("status-") && node.chapterId) {
      const status = key.replace("status-", "") as "draft" | "writing" | "final";
      await chapterStore.updateChapterMeta(node.chapterId, { status });
      message.success("状态已更新");
    }

    if (key === "delete-ch" && node.chapterId) {
      dialog.warning({
        title: "确认删除",
        content: `确定删除章节「${node.label}」吗？`,
        positiveText: "删除",
        negativeText: "取消",
        onPositiveClick: async () => {
          await chapterStore.deleteChapter(node.chapterId!);
          message.success("已删除");
        },
      });
    }
  } catch (e: any) {
    message.error(e?.message || "操作失败");
  }
}

// ---- 新建卷 ----
async function addVolume() {
  const title = prompt("输入卷名：", `第 ${projectStore.volumes.length + 1} 卷`);
  if (!title?.trim()) return;
  try {
    await projectStore.createVolume(title.trim());
    message.success("卷已创建");
  } catch (e: any) {
    message.error(e?.message || "创建失败");
  }
}
</script>

<template>
  <div class="chapter-tree flex flex-col h-full text-gray-800 dark:text-gray-200">
    <!-- 标题栏 -->
    <div class="flex items-center justify-between px-3 py-2 border-b border-gray-200 dark:border-gray-700">
      <span class="text-sm font-semibold">章节目录</span>
      <NButton size="tiny" quaternary @click="addVolume">
        <template #icon>
          <span class="i-carbon-add text-sm" />
        </template>
      </NButton>
    </div>

    <!-- 树 -->
    <div class="flex-1 overflow-auto py-1">
      <NEmpty
        v-if="projectStore.volumes.length === 0"
        description="暂无卷，点击 + 新建"
        class="py-8"
        size="small"
      />
      <NTree
        v-else
        :data="treeData"
        :selected-keys="selectedKeys"
        block-line
        expand-on-click
        selectable
        :default-expand-all="true"
        @update:selected-keys="handleSelect"
        @update:contextmenu="handleRightClick"
      />
    </div>

    <!-- 右键菜单 -->
    <NDropdown
      :show="contextmenu"
      :x="contextX"
      :y="contextY"
      :options="contextNode ? getContextMenu(contextNode) : []"
      placement="bottom-start"
      @select="handleContextSelect"
      @clickoutside="contextmenu = false"
    />
  </div>
</template>
