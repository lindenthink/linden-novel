import { ref, computed } from "vue";
import * as settingsApi from "../api/settings";

// 模块级单例，跨组件共享
const autoSaveEnabled = ref(true);
let initialized = false;

export function useEditorSettings() {
  async function init() {
    if (initialized) return;
    initialized = true;
    const saved = await settingsApi.getSetting("auto_save_enabled");
    // 默认启用自动保存；显式保存为 "false" 时关闭
    autoSaveEnabled.value = saved === null ? true : saved !== "false";
  }

  async function setAutoSave(enabled: boolean) {
    autoSaveEnabled.value = enabled;
    await settingsApi.setSetting("auto_save_enabled", enabled ? "true" : "false");
  }

  const isAutoSaveEnabled = computed(() => autoSaveEnabled.value);

  return { isAutoSaveEnabled, autoSaveEnabled, setAutoSave, init };
}
