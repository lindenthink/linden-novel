<script setup lang="ts">
import { NConfigProvider, NMessageProvider, NDialogProvider } from "naive-ui";
import { useTheme } from "./composables/useTheme";
import { useEditorSettings } from "./composables/useEditorSettings";
import FloatingSettings from "./components/common/FloatingSettings.vue";

const { theme, init: initTheme } = useTheme();
const { init: initEditorSettings } = useEditorSettings();

initTheme();
initEditorSettings();
</script>

<template>
  <NConfigProvider :theme="theme">
    <NMessageProvider>
      <NDialogProvider>
        <router-view />
        <FloatingSettings />
      </NDialogProvider>
    </NMessageProvider>
  </NConfigProvider>
</template>
