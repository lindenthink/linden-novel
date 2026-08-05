<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { NButton, NInput } from "naive-ui";

const greetMsg = ref("");
const name = ref("");

async function greet() {
  greetMsg.value = await invoke("greet", { name: name.value });
}
</script>

<template>
  <main class="min-h-screen flex flex-col items-center justify-center gap-4 p-8">
    <h1 class="text-2xl font-bold">Linden Novel · P0 脚手架</h1>
    <p class="text-gray-500 text-sm">
      Tauri v2 + Vue3 + Pinia + Router + Naive UI + UnoCSS
    </p>
    <div class="flex gap-2 items-center">
      <NInput v-model:value="name" placeholder="输入名字测试 greet..." class="w-64" />
      <NButton type="primary" @click="greet">Greet</NButton>
    </div>
    <p class="text-indigo-600">{{ greetMsg }}</p>
  </main>
</template>
