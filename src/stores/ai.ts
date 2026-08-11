import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type {
  AiProvider,
  AiApiKey,
  CompleteRequest,
  StreamChunkEvent,
} from "../types/ai";
import * as aiApi from "../api/ai";
import { listen } from "@tauri-apps/api/event";

export const useAiStore = defineStore("ai", () => {
  // ---- State ----
  const providers = ref<AiProvider[]>([]);
  const apiKeys = ref<Record<string, AiApiKey[]>>({});
  const defaultProvider = ref<AiProvider | null>(null);
  const loading = ref(false);
  const error = ref<string | null>(null);

  // 流式响应状态
  const streaming = ref(false);
  const streamContent = ref("");
  const streamError = ref<string | null>(null);

  // ---- Getters ----
  const providerList = computed(() => providers.value);
  const currentDefaultProvider = computed(() => defaultProvider.value);
  const isStreaming = computed(() => streaming.value);
  const currentStreamContent = computed(() => streamContent.value);

  // ---- Actions ----

  // Provider 管理
  async function loadProviders() {
    loading.value = true;
    error.value = null;
    try {
      providers.value = await aiApi.listAiProviders();
      defaultProvider.value = await aiApi.getDefaultAiProvider();
    } catch (e) {
      error.value = String(e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  async function createProvider(input: Omit<AiProvider, "id" | "created_at" | "updated_at">) {
    error.value = null;
    try {
      const provider = await aiApi.createAiProvider(input);
      if (provider.is_default) {
        providers.value.forEach((p) => {
          if (p.id !== provider.id) p.is_default = false;
        });
        defaultProvider.value = provider;
      }
      providers.value.push(provider);
      return provider;
    } catch (e) {
      error.value = String(e);
      throw e;
    }
  }

  async function updateProvider(id: string, input: Partial<AiProvider>) {
    error.value = null;
    try {
      const provider = await aiApi.updateAiProvider(id, input);
      if (provider.is_default) {
        providers.value.forEach((p) => {
          if (p.id !== provider.id) p.is_default = false;
        });
        defaultProvider.value = provider;
      }
      const index = providers.value.findIndex((p) => p.id === id);
      if (index !== -1) {
        providers.value[index] = provider;
      }
      return provider;
    } catch (e) {
      error.value = String(e);
      throw e;
    }
  }

  async function deleteProvider(id: string) {
    error.value = null;
    try {
      await aiApi.deleteAiProvider(id);
      providers.value = providers.value.filter((p) => p.id !== id);
      if (defaultProvider.value?.id === id) {
        defaultProvider.value = null;
      }
    } catch (e) {
      error.value = String(e);
      throw e;
    }
  }

  // API Key 管理
  async function loadApiKeys(providerId: string) {
    error.value = null;
    try {
      const keys = await aiApi.listAiApiKeys(providerId);
      apiKeys.value[providerId] = keys;
    } catch (e) {
      error.value = String(e);
      throw e;
    }
  }

  async function createApiKey(input: { provider_id: string; name: string; api_key: string; is_default?: boolean }) {
    error.value = null;
    try {
      const key = await aiApi.createAiApiKey(input);
      if (!apiKeys.value[input.provider_id]) {
        apiKeys.value[input.provider_id] = [];
      }
      if (key.is_default) {
        apiKeys.value[input.provider_id].forEach((k) => {
          if (k.id !== key.id) k.is_default = false;
        });
      }
      apiKeys.value[input.provider_id].push(key);
      return key;
    } catch (e) {
      error.value = String(e);
      throw e;
    }
  }

  async function deleteApiKey(id: string, providerId: string) {
    error.value = null;
    try {
      await aiApi.deleteAiApiKey(id);
      if (apiKeys.value[providerId]) {
        apiKeys.value[providerId] = apiKeys.value[providerId].filter((k) => k.id !== id);
      }
    } catch (e) {
      error.value = String(e);
      throw e;
    }
  }

  async function setDefaultApiKey(id: string, providerId: string) {
    error.value = null;
    try {
      await aiApi.setDefaultAiApiKey(id);
      if (apiKeys.value[providerId]) {
        apiKeys.value[providerId] = apiKeys.value[providerId].map((k) => ({
          ...k,
          is_default: k.id === id,
        }));
      }
    } catch (e) {
      error.value = String(e);
      throw e;
    }
  }

  // AI 补全
  async function complete(request: CompleteRequest) {
    error.value = null;
    try {
      return await aiApi.aiComplete(request);
    } catch (e) {
      error.value = String(e);
      throw e;
    }
  }

  // 流式 AI 补全
  async function completeStream(request: CompleteRequest): Promise<string> {
    if (streaming.value) {
      throw new Error("已有流式请求正在进行中");
    }
    error.value = null;
    streamError.value = null;
    streamContent.value = "";
    streaming.value = true;

    // 监听流式事件
    const unlistenChunk = await listen<StreamChunkEvent>("ai-stream-chunk", (event) => {
      streamContent.value += event.payload.content;
      if (event.payload.done) {
        streaming.value = false;
      }
    });

    const unlistenError = await listen<string>("ai-stream-error", (event) => {
      streamError.value = event.payload;
      streaming.value = false;
    });

    const unlistenDone = await listen("ai-stream-done", () => {
      streaming.value = false;
    });

    try {
      await aiApi.aiCompleteStream(request);
      return streamContent.value;
    } catch (e) {
      error.value = String(e);
      streamError.value = String(e);
      throw e;
    } finally {
      unlistenChunk();
      unlistenError();
      unlistenDone();
    }
  }

  return {
    // State
    providers,
    apiKeys,
    defaultProvider,
    loading,
    error,
    streaming,
    streamContent,
    streamError,

    // Getters
    providerList,
    currentDefaultProvider,
    isStreaming,
    currentStreamContent,

    // Actions
    loadProviders,
    createProvider,
    updateProvider,
    deleteProvider,
    loadApiKeys,
    createApiKey,
    deleteApiKey,
    setDefaultApiKey,
    complete,
    completeStream,
  };
});
