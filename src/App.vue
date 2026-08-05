<script setup lang="ts">
import { NConfigProvider, NMessageProvider, NDialogProvider } from "naive-ui";
import { useTheme } from "./composables/useTheme";

const { theme, init } = useTheme();
init();
</script>

<template>
  <NConfigProvider :theme="theme">
    <NMessageProvider>
      <NDialogProvider>
        <router-view />
      </NDialogProvider>
    </NMessageProvider>
  </NConfigProvider>
</template>
