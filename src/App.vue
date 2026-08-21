<script setup lang="ts">
import { onMounted } from "vue";
import { NConfigProvider, NMessageProvider, NDialogProvider } from "naive-ui";
import { useTheme } from "./composables/useTheme";
import { useEditorSettings } from "./composables/useEditorSettings";
import { useUpdater } from "./composables/useUpdater";
import FloatingSettings from "./components/common/FloatingSettings.vue";

const { theme, init: initTheme } = useTheme();
const { init: initEditorSettings } = useEditorSettings();
const { checkForUpdates } = useUpdater();
import { dateZhCN, zhCN } from 'naive-ui'


initTheme();
initEditorSettings();

// 启动 5s 后台静默检查更新（24h 节流，新版才弹 dialog，由 useUpdater 内 showDialog 控制）
onMounted(() => {
  setTimeout(() => {
    checkForUpdates(true).catch((e) =>
      console.warn("[updater] startup check failed:", e),
    );
  }, 5000);
});
</script>

<template>
  <NConfigProvider :theme="theme" :locale="zhCN" :date-locale="dateZhCN">
    <NMessageProvider>
      <NDialogProvider>
        <router-view />
        <FloatingSettings />
      </NDialogProvider>
    </NMessageProvider>
  </NConfigProvider>
</template>
