<script setup lang="ts">
import { ref, computed, onMounted, watch, h } from "vue";
import {
  NModal,
  NTabs,
  NTabPane,
  NButton,
  NSpace,
  NForm,
  NFormItem,
  NInput,
  NSelect,
  NSwitch,
  NDataTable,
  NPopconfirm,
  NEmpty,
  NSpin,
  NCard,
  useMessage,
} from "naive-ui";
import type { DataTableColumns } from "naive-ui";
import { useAiStore } from "../../stores/ai";
import type { AiProvider, AiApiKey } from "../../types/ai";

defineProps<{
  show: boolean;
}>();

const emit = defineEmits<{
  "update:show": [value: boolean];
}>();

const aiStore = useAiStore();
const message = useMessage();

const activeTab = ref<"provider" | "apikey" | "narrative-rules">("provider");

const providerTypeOptions = [
  { label: "OpenAI", value: "openai" },
  { label: "DeepSeek", value: "deepseek" },
  { label: "Claude / Anthropic", value: "claude" },
];

// Provider 预设配置
const providerPresets: Record<string, { base_url: string; model: string; embedding_model?: string }> = {
  openai: { base_url: "https://api.openai.com", model: "gpt-4o" },
  deepseek: { base_url: "https://api.deepseek.com", model: "deepseek-v4-flash" },
  claude: { base_url: "https://api.anthropic.com", model: "claude-3-5-sonnet" },
};

// ---- Provider 管理 ----
const showProviderForm = ref(false);
const editingProvider = ref<AiProvider | null>(null);
const providerForm = ref({
  name: "",
  provider_type: "openai",
  base_url: "https://api.openai.com",
  models_json: "gpt-4o",
  is_default: false,
});

function openProviderForm() {
  resetProviderForm();
  showProviderForm.value = true;
}

// 类型切换时自动填充预设（仅新建时）
watch(
  () => providerForm.value.provider_type,
  (type) => {
    if (editingProvider.value) return; // 编辑时不自动覆盖
    const preset = providerPresets[type];
    if (preset) {
      providerForm.value.base_url = preset.base_url;
      providerForm.value.models_json = preset.model;
    }
  }
);

function resetProviderForm() {
  providerForm.value = {
    name: "",
    provider_type: "deepseek",
    base_url: "https://api.deepseek.com",
    models_json: "deepseek-v4-flash",
    is_default: false,
  };
  editingProvider.value = null;
}

function editProvider(provider: AiProvider) {
  editingProvider.value = provider;
  providerForm.value = {
    name: provider.name,
    provider_type: provider.provider_type,
    base_url: provider.base_url,
    models_json: provider.models_json,
    is_default: provider.is_default,
  };
  showProviderForm.value = true;
}

async function saveProvider() {
  try {
    if (editingProvider.value) {
      await aiStore.updateProvider(editingProvider.value.id, providerForm.value);
      message.success("Provider 更新成功");
    } else {
      await aiStore.createProvider(providerForm.value);
      message.success("Provider 创建成功");
    }
    showProviderForm.value = false;
    resetProviderForm();
  } catch (e: any) {
    message.error(e?.message || "保存失败");
  }
}

async function deleteProvider(id: string) {
  try {
    await aiStore.deleteProvider(id);
    message.success("Provider 已删除");
  } catch (e: any) {
    message.error(e?.message || "删除失败");
  }
}

const providerColumns: DataTableColumns<AiProvider> = [
  {
    title: "名称",
    key: "name",
    width: 150,
  },
  {
    title: "类型",
    key: "provider_type",
    width: 100,
  },
  {
    title: "Base URL",
    key: "base_url",
    ellipsis: {
      tooltip: true,
    },
  },
  {
    title: "默认",
    key: "is_default",
    width: 80,
    render(row) {
      return row.is_default ? "✓" : "";
    },
  },
  {
    title: "操作",
    key: "actions",
    width: 150,
    render(row) {
      return [
        h(
          NButton,
          {
            size: "small",
            quaternary: true,
            onClick: () => editProvider(row),
          },
          { default: () => "编辑" }
        ),
        h(
          NPopconfirm,
          {
            onPositiveClick: () => deleteProvider(row.id),
          },
          {
            trigger: () =>
              h(
                NButton,
                {
                  size: "small",
                  quaternary: true,
                  type: "error",
                },
                { default: () => "删除" }
              ),
            default: () => "确定删除吗？",
          }
        ),
      ];
    },
  },
];

// ---- API Key 管理 ----
const selectedProviderId = ref<string>("");

const providerOptions = computed(() =>
  aiStore.providers.map((p) => ({
    label: p.name,
    value: p.id,
  }))
);

async function handleProviderChange(id: string) {
  selectedProviderId.value = id;
  await aiStore.loadApiKeys(id);
}

const showApiKeyForm = ref(false);
const apikeyForm = ref({
  provider_id: "",
  name: "",
  api_key: "",
  is_default: false,
});

function resetApiKeyForm() {
  apikeyForm.value = {
    provider_id: selectedProviderId.value,
    name: "",
    api_key: "",
    is_default: false,
  };
}

// 监听 provider 切换，同步 API Key 表单的 provider_id
watch(
  () => selectedProviderId.value,
  (id) => {
    if (id) {
      apikeyForm.value.provider_id = id;
    }
  }
);

function openApiKeyForm() {
  resetApiKeyForm();
  showApiKeyForm.value = true;
}

async function saveApiKey() {
  try {
    await aiStore.createApiKey(apikeyForm.value);
    message.success("API Key 创建成功");
    showApiKeyForm.value = false;
    resetApiKeyForm();
    if (selectedProviderId.value) {
      await aiStore.loadApiKeys(selectedProviderId.value);
    }
  } catch (e: any) {
    message.error(e?.message || "保存失败");
  }
}

async function deleteApiKey(id: string) {
  try {
    await aiStore.deleteApiKey(id, selectedProviderId.value);
    message.success("API Key 已删除");
    if (selectedProviderId.value) {
      await aiStore.loadApiKeys(selectedProviderId.value);
    }
  } catch (e: any) {
    message.error(e?.message || "删除失败");
  }
}

async function setDefaultApiKey(id: string) {
  try {
    await aiStore.setDefaultApiKey(id, selectedProviderId.value);
    message.success("已设为默认");
    if (selectedProviderId.value) {
      await aiStore.loadApiKeys(selectedProviderId.value);
    }
  } catch (e: any) {
    message.error(e?.message || "设置失败");
  }
}

const apikeyColumns: DataTableColumns<AiApiKey> = [
  {
    title: "名称",
    key: "name",
    width: 200,
  },
  {
    title: "默认",
    key: "is_default",
    width: 80,
    render(row) {
      return row.is_default ? "✓" : "";
    },
  },
  {
    title: "创建时间",
    key: "created_at",
    render(row) {
      return new Date(row.created_at).toLocaleString();
    },
  },
  {
    title: "操作",
    key: "actions",
    width: 200,
    render(row) {
      return [
        !row.is_default &&
          h(
            NButton,
            {
              size: "small",
              quaternary: true,
              onClick: () => setDefaultApiKey(row.id),
            },
            { default: () => "设为默认" }
          ),
        h(
          NPopconfirm,
          {
            onPositiveClick: () => deleteApiKey(row.id),
          },
          {
            trigger: () =>
              h(
                NButton,
                {
                  size: "small",
                  quaternary: true,
                  type: "error",
                },
                { default: () => "删除" }
              ),
            default: () => "确定删除吗？",
          }
        ),
      ];
    },
  },
];

// ---- 叙事规则模板管理（prompt_templates: narrative_loose / narrative_strict）----
const BUILTIN_LOOSE_ID = "builtin-narrative-loose";
const BUILTIN_STRICT_ID = "builtin-narrative-strict";

const narrativeSaving = ref<string | null>(null); // 'loose' | 'strict' | null

const looseTemplate = computed(() =>
  aiStore.promptTemplates.find((t) => t.id === BUILTIN_LOOSE_ID)
);
const strictTemplate = computed(() =>
  aiStore.promptTemplates.find((t) => t.id === BUILTIN_STRICT_ID)
);

// 本地内容缓存：用户在 textarea 内编辑后不直接回写 store，点「保存」再提交
const looseContentLocal = ref("");
const strictContentLocal = ref("");

// 当 store 中的模板变化（加载/重置）时，同步本地缓存
watch(
  () => looseTemplate.value?.content,
  (c) => {
    if (typeof c === "string") looseContentLocal.value = c;
  },
  { immediate: true }
);
watch(
  () => strictTemplate.value?.content,
  (c) => {
    if (typeof c === "string") strictContentLocal.value = c;
  },
  { immediate: true }
);

async function saveNarrativeTemplate(kind: "loose" | "strict") {
  const id = kind === "loose" ? BUILTIN_LOOSE_ID : BUILTIN_STRICT_ID;
  const localContent =
    kind === "loose" ? looseContentLocal.value : strictContentLocal.value;
  narrativeSaving.value = kind;
  try {
    if (!localContent.trim()) {
      message.warning("模板内容不能为空");
      return;
    }
    await aiStore.updatePromptTemplate(id, { content: localContent });
    message.success(`${kind === "loose" ? "宽松" : "严格"}规则保存成功`);
  } catch (e: any) {
    message.error(e?.message || "保存失败");
  } finally {
    narrativeSaving.value = null;
  }
}

async function resetNarrativeTemplate(kind: "loose" | "strict") {
  const id = kind === "loose" ? BUILTIN_LOOSE_ID : BUILTIN_STRICT_ID;
  narrativeSaving.value = kind;
  try {
    const t = await aiStore.resetPromptTemplateBuiltin(id);
    // reset 后 store 已更新，watch 会同步本地缓存
    message.success(
      `${kind === "loose" ? "宽松" : "严格"}规则已恢复为默认：${t.name}`
    );
  } catch (e: any) {
    message.error(e?.message || "恢复默认失败");
  } finally {
    narrativeSaving.value = null;
  }
}

// 监听 tab 切换，加载对应数据
function handleTabChange(key: string) {
  if (key === "apikey" && selectedProviderId.value) {
    aiStore.loadApiKeys(selectedProviderId.value);
  }
  if (key === "narrative-rules") {
    aiStore.loadPromptTemplates();
  }
}

onMounted(async () => {
  await aiStore.loadProviders();
  if (aiStore.providers.length > 0) {
    selectedProviderId.value = aiStore.providers[0].id;
    apikeyForm.value.provider_id = selectedProviderId.value;
    await aiStore.loadApiKeys(selectedProviderId.value);
  }
});
</script>

<template>
  <NModal
    :show="show"
    @update:show="emit('update:show', $event)"
    preset="card"
    title="AI 设置"
    style="width: 900px; max-width: 90vw"
  >
    <NTabs v-model:value="activeTab" type="line" @update:value="handleTabChange">
      <!-- Provider 管理 -->
      <NTabPane name="provider" tab="Provider 管理">
        <NSpace vertical>
          <NButton type="primary" @click="openProviderForm">
            <template #icon>
              <span class="i-carbon-add" />
            </template>
            添加 Provider
          </NButton>

          <NDataTable
            :columns="providerColumns"
            :data="aiStore.providers"
            :bordered="false"
            :single-line="false"
            size="small"
          />

          <!-- Provider 表单 -->
          <NModal v-model:show="showProviderForm" preset="dialog" title="Provider" positive-text="保存" negative-text="取消" @positive-click="saveProvider" @negative-click="resetProviderForm" style="width: 600px">
            <NForm>
              <NFormItem label="名称">
                <NInput v-model:value="providerForm.name" placeholder="例如：Deepseek" />
              </NFormItem>
              <NFormItem label="类型">
                <NSelect
                  v-model:value="providerForm.provider_type"
                  :options="providerTypeOptions"
                  placeholder="请选择类型"
                />
              </NFormItem>
              <NFormItem label="Base URL">
                <NInput v-model:value="providerForm.base_url" placeholder="https://api.openai.com" />
              </NFormItem>
              <NFormItem label="模型名称">
                <NInput v-model:value="providerForm.models_json" placeholder="例如：gpt-4o / deepseek-chat" />
              </NFormItem>
              <NFormItem label="设为默认">
                <NSwitch v-model:value="providerForm.is_default" />
              </NFormItem>
            </NForm>
          </NModal>
        </NSpace>
      </NTabPane>

      <!-- API Key 管理 -->
      <NTabPane name="apikey" tab="API Key 管理">
        <NSpace vertical>
          <NForm inline>
            <NFormItem label="选择 Provider">
              <NSelect
                :value="selectedProviderId"
                :options="providerOptions"
                placeholder="请选择 Provider"
                :style="{ width: '300px' }"
                @update:value="handleProviderChange"
              />
            </NFormItem>
          </NForm>

          <NButton type="primary" @click="openApiKeyForm" :disabled="!selectedProviderId">
            <template #icon>
              <span class="i-carbon-add" />
            </template>
            添加 API Key
          </NButton>

          <NDataTable
            v-if="selectedProviderId && aiStore.apiKeys[selectedProviderId]"
            :columns="apikeyColumns"
            :data="aiStore.apiKeys[selectedProviderId]"
            :bordered="false"
            :single-line="false"
            size="small"
          />
          <NEmpty v-else description="请选择 Provider 或暂无 API Key" />

          <!-- API Key 表单 -->
          <NModal v-model:show="showApiKeyForm" preset="dialog" title="API Key" positive-text="保存" negative-text="取消" @positive-click="saveApiKey" @negative-click="resetApiKeyForm" style="width: 500px">
            <NForm>
              <NFormItem label="名称">
                <NInput v-model:value="apikeyForm.name" placeholder="例如：我的 OpenAI Key" />
              </NFormItem>
              <NFormItem label="API Key">
                <NInput v-model:value="apikeyForm.api_key" type="password" show-password-on="click" placeholder="sk-..." />
              </NFormItem>
              <NFormItem label="设为默认">
                <NSwitch v-model:value="apikeyForm.is_default" />
              </NFormItem>
            </NForm>
          </NModal>
        </NSpace>
      </NTabPane>

      <!-- 叙事规则模板管理 -->
      <NTabPane name="narrative-rules" tab="叙事规则">
        <NSpin :show="aiStore.promptTemplatesLoading">
          <NSpace vertical style="gap: 16px">
            <div style="font-size: 13px; color: #666;">
              这里的模板会在 AI 生成章节时作为「约束程度」的默认规则使用。用户可根据写作习惯自由修改，修改后立即生效。
            </div>

            <!-- 宽松 -->
            <NCard :bordered="true" size="small">
              <template #header>
                <NSpace align="center" justify="space-between" style="width: 100%">
                  <span style="font-weight: 500">约束程度：宽松</span>
                  <NSpace>
                    <NPopconfirm
                      @positive-click="() => resetNarrativeTemplate('loose')"
                    >
                      <template #trigger>
                        <NButton
                          size="small"
                          :disabled="narrativeSaving !== null"
                        >
                          恢复默认
                        </NButton>
                      </template>
                      确定要将「宽松」规则恢复为默认内容吗？当前修改将会丢失。
                    </NPopconfirm>
                    <NButton
                      size="small"
                      type="primary"
                      :loading="narrativeSaving === 'loose'"
                      @click="() => saveNarrativeTemplate('loose')"
                    >
                      保存
                    </NButton>
                  </NSpace>
                </NSpace>
              </template>
              <NInput
                v-model:value="looseContentLocal"
                type="textarea"
                :autosize="{ minRows: 8, maxRows: 16 }"
                placeholder="例如：保持与前文一致的叙事视角和语言风格..."
              />
            </NCard>

            <!-- 严格 -->
            <NCard :bordered="true" size="small">
              <template #header>
                <NSpace align="center" justify="space-between" style="width: 100%">
                  <span style="font-weight: 500">约束程度：严格</span>
                  <NSpace>
                    <NPopconfirm
                      @positive-click="() => resetNarrativeTemplate('strict')"
                    >
                      <template #trigger>
                        <NButton
                          size="small"
                          :disabled="narrativeSaving !== null"
                        >
                          恢复默认
                        </NButton>
                      </template>
                      确定要将「严格」规则恢复为默认内容吗？当前修改将会丢失。
                    </NPopconfirm>
                    <NButton
                      size="small"
                      type="primary"
                      :loading="narrativeSaving === 'strict'"
                      @click="() => saveNarrativeTemplate('strict')"
                    >
                      保存
                    </NButton>
                  </NSpace>
                </NSpace>
              </template>
              <NInput
                v-model:value="strictContentLocal"
                type="textarea"
                :autosize="{ minRows: 8, maxRows: 20 }"
                placeholder="例如：视角：紧贴主角，不写他不知道的事..."
              />
            </NCard>
          </NSpace>
        </NSpin>
      </NTabPane>
    </NTabs>
  </NModal>
</template>
