import { ref, computed } from "vue";
import { darkTheme } from "naive-ui";
import type { GlobalTheme } from "naive-ui";
import * as settingsApi from "../api/settings";

// 模块级单例，跨组件共享
const isDark = ref(false);
let initialized = false;

export function useTheme() {
  async function init() {
    if (initialized) return;
    initialized = true;
    const saved = await settingsApi.getSetting("theme");
    if (saved === "dark") isDark.value = true;
    applyClass();
  }

  async function toggle() {
    isDark.value = !isDark.value;
    applyClass();
    await settingsApi.setSetting("theme", isDark.value ? "dark" : "light");
  }

  function applyClass() {
    document.documentElement.classList.toggle("dark", isDark.value);
  }

  const theme = computed<GlobalTheme | null>(() => (isDark.value ? darkTheme : null));

  return { isDark, theme, toggle, init };
}
