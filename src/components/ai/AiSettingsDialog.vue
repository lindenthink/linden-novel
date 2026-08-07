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
  useMessage,
} from "naive-ui";
import type { DataTableColumns } from "naive-ui";
import { useAiStore } from "../../stores/ai";
import type { AiProvider, AiApiKey, PromptTemplate } from "../../types/ai";

defineProps<{
  show: boolean;
}>();

const emit = defineEmits<{
  "update:show": [value: boolean];
}>();

const aiStore = useAiStore();
const message = useMessage();

const activeTab = ref<"provider" | "apikey" | "template">("provider");

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
    provider_type: "openai",
    base_url: "https://api.openai.com",
    models_json: "gpt-4o",
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

// ---- Prompt Template 管理 ----
const showTemplateForm = ref(false);
const editingTemplate = ref<PromptTemplate | null>(null);
const templateForm = ref({
  name: "",
  template_type: "completion",
  content: "",
  variables_json: "[]",
});

function resetTemplateForm() {
  templateForm.value = {
    name: "",
    template_type: "completion",
    content: "",
    variables_json: "[]",
  };
  editingTemplate.value = null;
}

function editTemplate(template: PromptTemplate) {
  editingTemplate.value = template;
  templateForm.value = {
    name: template.name,
    template_type: template.template_type,
    content: template.content,
    variables_json: template.variables_json,
  };
  showTemplateForm.value = true;
}

async function saveTemplate() {
  try {
    if (editingTemplate.value) {
      await aiStore.updateTemplate(editingTemplate.value.id, templateForm.value);
      message.success("模板更新成功");
    } else {
      await aiStore.createTemplate(templateForm.value);
      message.success("模板创建成功");
    }
    showTemplateForm.value = false;
    resetTemplateForm();
  } catch (e: any) {
    message.error(e?.message || "保存失败");
  }
}

async function deleteTemplate(id: string) {
  try {
    await aiStore.deleteTemplate(id);
    message.success("模板已删除");
  } catch (e: any) {
    message.error(e?.message || "删除失败");
  }
}

const templateColumns: DataTableColumns<PromptTemplate> = [
  {
    title: "名称",
    key: "name",
    width: 200,
  },
  {
    title: "类型",
    key: "template_type",
    width: 120,
  },
  {
    title: "内容预览",
    key: "content",
    ellipsis: {
      tooltip: true,
    },
    render(row) {
      return row.content.substring(0, 50) + (row.content.length > 50 ? "..." : "");
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
            onClick: () => editTemplate(row),
          },
          { default: () => "编辑" }
        ),
        h(
          NPopconfirm,
          {
            onPositiveClick: () => deleteTemplate(row.id),
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

// 监听 tab 切换，加载对应数据
function handleTabChange(key: string) {
  if (key === "apikey" && selectedProviderId.value) {
    aiStore.loadApiKeys(selectedProviderId.value);
  }
}

onMounted(async () => {
  await aiStore.loadProviders();
  await aiStore.loadTemplates();
  if (aiStore.providers.length > 0) {
    selectedProviderId.value = aiStore.providers[0].id;
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
          <NButton type="primary" @click="showProviderForm = true">
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
                <NInput v-model:value="providerForm.name" placeholder="例如：OpenAI" />
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

          <NButton type="primary" @click="showApiKeyForm = true" :disabled="!selectedProviderId">
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

      <!-- Prompt Template 管理 -->
      <NTabPane name="template" tab="Prompt 模板">
        <NSpace vertical>
          <NButton type="primary" @click="showTemplateForm = true">
            <template #icon>
              <span class="i-carbon-add" />
            </template>
            添加模板
          </NButton>

          <NDataTable
            :columns="templateColumns"
            :data="aiStore.templates"
            :bordered="false"
            :single-line="false"
            size="small"
          />

          <!-- Template 表单 -->
          <NModal v-model:show="showTemplateForm" preset="dialog" title="Prompt 模板" positive-text="保存" negative-text="取消" @positive-click="saveTemplate" @negative-click="resetTemplateForm" style="width: 700px">
            <NForm>
              <NFormItem label="名称">
                <NInput v-model:value="templateForm.name" placeholder="例如：续写助手" />
              </NFormItem>
              <NFormItem label="类型">
                <NInput v-model:value="templateForm.template_type" placeholder="completion / continuation / summary" />
              </NFormItem>
              <NFormItem label="模板内容">
                <NInput v-model:value="templateForm.content" type="textarea" :rows="8" placeholder="使用 {{variable}} 格式的变量" />
              </NFormItem>
              <NFormItem label="变量列表 (JSON)">
                <NInput v-model:value="templateForm.variables_json" type="textarea" :rows="2" placeholder='["context", "style"]' />
              </NFormItem>
            </NForm>
          </NModal>
        </NSpace>
      </NTabPane>
    </NTabs>
  </NModal>
</template>
