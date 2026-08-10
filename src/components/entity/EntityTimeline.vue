<script setup lang="ts">
import { computed, ref, onMounted, watch } from "vue";
import {
  NEmpty,
  NTabs,
  NTabPane,
  NTag,
  NButton,
  NButtonGroup,
  NSpin,
  useMessage,
} from "naive-ui";
import { useEntitySnapshot } from "../../composables/useEntitySnapshot";
import { useChapterStore } from "../../stores/chapter";

const props = defineProps<{
  projectId: string;
}>();

const message = useMessage();
const chapterStore = useChapterStore();
const {
  evolutionLoading,
  currentEvolution,
  chapterLoading,
  chapterSnapshots,
  projectEntities,
  generating,
  generateResult,
  fetchEvolution,
  fetchChapterSnapshots,
  fetchProjectEntities,
  handleGenerateSnapshots,
  handleBatchSnapshots,
  resetEvolution,
} = useEntitySnapshot();

// 标签页: "chapter" (本章快照) | "evolution" (实体演变)
const activeTab = ref("chapter");
const selectedEntityType = ref<"character" | "storyline">("character");
const selectedEntityId = ref<string | null>(null);

// 项目级实体列表（所有有快照的实体，不限于当前章节）
const availableEntities = computed(() => {
  return projectEntities.value
    .filter((e) => e.entity_type === selectedEntityType.value)
    .map((e) => ({ id: e.entity_id, name: e.name }));
});

function extractName(s: { summary: string }) {
  // 从摘要中提取名称: "张三（状态：alive）"
  const match = s.summary.match(/^(.+?)（/);
  return match ? match[1] : s.summary.slice(0, 20);
}

// 切换实体类型时重置
watch(selectedEntityType, () => {
  selectedEntityId.value = null;
  resetEvolution();
});

// 选择实体加载演变历史
async function selectEntity(entityId: string) {
  selectedEntityId.value = entityId;
  await fetchEvolution(selectedEntityType.value, entityId);
}

// 生成当前章节快照
async function onGenerateCurrent() {
  const chapterId = chapterStore.activeChapterId;
  if (!chapterId) {
    message.warning("请先选择章节");
    return;
  }
  try {
    const res = await handleGenerateSnapshots(chapterId, props.projectId);
    message.success(`快照生成完成：成功 ${res.success_count}，失败 ${res.failed_count}`);
  } catch (e: any) {
    message.error(e?.toString() || "生成失败");
  }
}

// 批量生成
async function onBatchGenerate() {
  if (!props.projectId) return;
  try {
    const res = await handleBatchSnapshots(props.projectId);
    message.success(`批量完成：成功 ${res.success_count}，失败 ${res.failed_count}`);
  } catch (e: any) {
    message.error(e?.toString() || "批量生成失败");
  }
}

// 解析状态 JSON 为可读文本
function formatStateJson(jsonStr: string): { key: string; value: string }[] {
  try {
    const obj = JSON.parse(jsonStr);
    return Object.entries(obj)
      .filter(([_, v]) => v !== null && v !== "" && (!Array.isArray(v) || v.length > 0))
      .map(([k, v]) => ({
        key: translateKey(k),
        value: formatValue(v),
      }));
  } catch {
    return [{ key: "raw", value: jsonStr }];
  }
}

function translateKey(key: string): string {
  const map: Record<string, string> = {
    status: "状态",
    progress: "进展",
    location: "位置",
    role_change: "身份变化",
    relationships: "关系变化",
    key_events: "关键事件",
    key_developments: "关键进展",
    emotional_state: "情感状态",
    involved_characters: "涉及角色",
    foreshadowing: "伏笔",
    tension_level: "紧张度",
  };
  return map[key] ?? key;
}

function formatValue(v: unknown): string {
  if (Array.isArray(v)) return v.join("、");
  if (typeof v === "object" && v !== null) {
    return Object.entries(v as Record<string, string>)
      .map(([k, val]) => `${k}: ${val}`)
      .join("；");
  }
  return String(v);
}

function statusTagType(status: string): "success" | "warning" | "error" | "info" | "default" {
  const map: Record<string, "success" | "warning" | "error" | "info" | "default"> = {
    alive: "success",
    dead: "error",
    missing: "warning",
    resolved: "success",
    advancing: "info",
    stalled: "warning",
    introduced: "default",
  };
  return map[status.toLowerCase()] ?? "default";
}

// 加载章节快照和项目实体
async function loadChapter() {
  const chapterId = chapterStore.activeChapterId;
  if (chapterId) {
    await fetchChapterSnapshots(chapterId);
  }
}

onMounted(async () => {
  await loadChapter();
  if (props.projectId) {
    await fetchProjectEntities(props.projectId);
  }
});

watch(() => chapterStore.activeChapterId, () => {
  loadChapter();
  selectedEntityId.value = null;
  resetEvolution();
});
</script>

<template>
  <div class="entity-timeline flex flex-col h-full text-gray-800 dark:text-gray-200">
    <!-- 操作栏 -->
    <div class="flex items-center gap-2 px-3 py-2 border-b border-gray-100 dark:border-gray-800">
      <NButton size="small" quaternary :loading="generating" @click="onGenerateCurrent">
        <template #icon>
          <span class="i-carbon-magic-wand" />
        </template>
        生成本章快照
      </NButton>
      <NButton size="small" quaternary :loading="generating" @click="onBatchGenerate">
        <template #icon>
          <span class="i-carbon-workspace" />
        </template>
        批量生成
      </NButton>
      <span v-if="generateResult" class="text-xs text-gray-500 ml-1">
        {{ generateResult }}
      </span>
    </div>

    <!-- 标签页 -->
    <NTabs
      v-model:value="activeTab"
      type="line"
      size="small"
      class="flex-1 flex flex-col overflow-hidden"
    >
      <!-- Tab 1: 本章快照 -->
      <NTabPane name="chapter" tab="本章状态" class="flex-1 overflow-auto">
        <div v-if="chapterLoading" class="flex justify-center py-10">
          <NSpin size="small" />
        </div>
        <NEmpty
          v-else-if="chapterSnapshots.length === 0"
          description="本章暂无实体快照，点击上方按钮生成"
          class="py-8"
          size="small"
        />
        <div v-else class="p-2 space-y-2">
          <div
            v-for="snap in chapterSnapshots"
            :key="snap.id"
            class="snapshot-card rounded-lg border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 p-3"
          >
            <div class="flex items-center gap-2 mb-2">
              <NTag
                :type="snap.entity_type === 'character' ? 'info' : 'warning'"
                size="small"
                round
              >
                {{ snap.entity_type === "character" ? "角色" : "故事线" }}
              </NTag>
              <span class="text-sm font-medium">
                {{ extractName(snap) }}
              </span>
            </div>

            <!-- 状态摘要 -->
            <p class="text-xs text-gray-600 dark:text-gray-300 mb-2">
              {{ snap.summary }}
            </p>

            <!-- 状态详情 -->
            <div
              v-if="formatStateJson(snap.state_json).length > 0"
              class="grid grid-cols-1 gap-1"
            >
              <div
                v-for="field in formatStateJson(snap.state_json)"
                :key="field.key"
                class="flex items-start gap-2 text-xs"
              >
                <span class="text-gray-400 shrink-0 w-16">{{ field.key }}</span>
                <span
                  v-if="field.key === '状态' || field.key === '进展'"
                  class="shrink-0"
                >
                  <NTag :type="statusTagType(field.value)" size="tiny" round>
                    {{ field.value }}
                  </NTag>
                </span>
                <span v-else class="text-gray-700 dark:text-gray-200 break-all">
                  {{ field.value }}
                </span>
              </div>
            </div>

            <!-- 变化标记 -->
            <div
              v-if="snap.changes"
              class="mt-2 text-xs text-amber-600 dark:text-amber-400 bg-amber-50 dark:bg-amber-900/20 px-2 py-1 rounded"
            >
              <span class="i-carbon-update text-xs" />
              {{ snap.changes }}
            </div>
          </div>
        </div>
      </NTabPane>

      <!-- Tab 2: 实体演变 -->
      <NTabPane name="evolution" tab="演变时间线" class="flex-1 overflow-hidden">
        <div class="flex h-full">
          <!-- 实体列表 -->
          <div class="w-36 border-r border-gray-100 dark:border-gray-800 overflow-auto">
            <div class="p-2 border-b border-gray-100 dark:border-gray-800">
              <NButtonGroup size="tiny" :vertical="false" class="w-full">
                <NButton
                  :type="selectedEntityType === 'character' ? 'primary' : 'default'"
                  @click="selectedEntityType = 'character'"
                  class="flex-1"
                >
                  角色
                </NButton>
                <NButton
                  :type="selectedEntityType === 'storyline' ? 'primary' : 'default'"
                  @click="selectedEntityType = 'storyline'"
                  class="flex-1"
                >
                  故事线
                </NButton>
              </NButtonGroup>
            </div>

            <div class="p-1 space-y-1">
              <button
                v-for="entity in availableEntities"
                :key="entity.id"
                :class="[
                  'w-full text-left text-sm px-2 py-1.5 rounded-md border transition',
                  selectedEntityId === entity.id
                    ? 'border-green-500 bg-green-50 dark:bg-green-900/40 text-green-800 dark:text-green-100 font-medium'
                    : 'border-transparent text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:bg-gray-800/30 dark:hover:bg-gray-700',
                ]"
                @click="selectEntity(entity.id)"
              >
                {{ entity.name }}
              </button>
              <div v-if="availableEntities.length === 0" class="p-3 text-xs text-gray-400 dark:text-gray-500 text-center">
                暂无{{ selectedEntityType === "character" ? "角色" : "故事线" }}快照，请先生成
              </div>
            </div>
          </div>

          <!-- 时间线 -->
          <div class="flex-1 overflow-auto p-3">
            <NSpin v-if="evolutionLoading" class="mt-10" />
            <NEmpty
              v-else-if="!currentEvolution"
              description="选择左侧实体查看演变时间线"
              class="py-8"
              size="small"
            />
            <template v-else>
              <!-- 实体名称 -->
              <div class="mb-3">
                <h3 class="text-base font-semibold">{{ currentEvolution.name }}</h3>
                <span class="text-xs text-gray-500">
                  共 {{ currentEvolution.snapshots.length }} 个快照
                </span>
              </div>

              <!-- 时间线 -->
              <div class="timeline relative">
                <!-- 时间线竖线 -->
                <div class="absolute left-2 top-2 bottom-2 w-px bg-gray-200 dark:bg-gray-700" />

                <div
                  v-for="(snap, idx) in currentEvolution.snapshots"
                  :key="snap.id"
                  class="relative pl-6 pb-4"
                >
                  <!-- 时间线节点圆点 -->
                  <div
                    :class="[
                      'absolute left-0 w-4 h-4 rounded-full border-2 flex items-center justify-center',
                      idx === currentEvolution.snapshots.length - 1
                        ? 'bg-green-500 border-green-600'
                        : 'bg-gray-200 dark:bg-gray-800 border-gray-300 dark:border-gray-600',
                    ]"
                  >
                    <div v-if="idx === currentEvolution.snapshots.length - 1" class="w-1.5 h-1.5 bg-white rounded-full" />
                  </div>

                  <!-- 章节标题 -->
                  <div class="text-xs font-medium text-gray-700 dark:text-gray-300 mb-1">
                    {{ snap.chapter_title }}
                  </div>

                  <!-- 状态卡片 -->
                  <div class="bg-gray-50 dark:bg-gray-800/50 rounded p-2">
                    <p class="text-xs text-gray-600 dark:text-gray-300 mb-1">
                      {{ snap.summary }}
                    </p>

                    <!-- 状态字段 -->
                    <div
                      v-if="formatStateJson(snap.state_json).length > 0"
                      class="flex flex-wrap gap-1"
                    >
                      <template
                        v-for="field in formatStateJson(snap.state_json)"
                        :key="field.key"
                      >
                        <NTag
                          v-if="field.key === '状态' || field.key === '进展'"
                          :type="statusTagType(field.value)"
                          size="tiny"
                          round
                        >
                          {{ field.value }}
                        </NTag>
                        <span v-else class="text-xs text-gray-500">
                          {{ field.key }}: {{ field.value }}
                        </span>
                      </template>
                    </div>

                    <!-- 变化 -->
                    <div
                      v-if="snap.changes"
                      class="mt-1.5 text-xs text-amber-600 dark:text-amber-400 flex items-center gap-1"
                    >
                      <span class="i-carbon-update text-xs" />
                      {{ snap.changes }}
                    </div>
                  </div>
                </div>
              </div>
            </template>
          </div>
        </div>
      </NTabPane>
    </NTabs>
  </div>
</template>

<style scoped>
.snapshot-card {
  transition: box-shadow 0.15s;
}
.snapshot-card:hover {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}
</style>
