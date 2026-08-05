<script setup lang="ts">
import { NConfigProvider, NMessageProvider } from "naive-ui";
</script>

<template>
  <NConfigProvider>
    <NMessageProvider>
      <router-view />
    </NMessageProvider>
  </NConfigProvider>
</template>
