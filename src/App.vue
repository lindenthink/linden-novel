<script setup lang="ts">
import { NConfigProvider, NMessageProvider, NDialogProvider } from "naive-ui";
import { useTheme } from "./composables/useTheme";
import FloatingSettings from "./components/common/FloatingSettings.vue";

const { theme, init } = useTheme();
init();
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
