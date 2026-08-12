<script setup lang="ts">
import { ref } from "vue";
import { NFloatButton, NTooltip } from "naive-ui";
import { useTheme } from "../../composables/useTheme";
import AiSettingsDialog from "../ai/AiSettingsDialog.vue";

const { toggle, isDark } = useTheme();
const showAiSettings = ref(false);
const showMenu = ref(false);

function handleAiSettings() {
  showMenu.value = false;
  showAiSettings.value = true;
}

function handleThemeToggle() {
  showMenu.value = false;
  toggle();
}
</script>

<template>
  <NFloatButton
    v-model:show-menu="showMenu"
    menu-trigger="click"
    :bottom="80"
    :right="40"
    shape="circle"
  >
    <span class="i-carbon-settings text-xl" />
    <template #menu>
      <NTooltip trigger="hover" placement="left">
        <template #trigger>
          <NFloatButton shape="circle" @click="handleAiSettings" >
            <span class="i-carbon-ai text-lg" />
          </NFloatButton>
        </template>
        AI 设置
      </NTooltip>
      <NTooltip trigger="hover" placement="left">
        <template #trigger>
          <NFloatButton shape="circle" @click="handleThemeToggle">
            <span
              :class="[isDark ? 'i-carbon-sun' : 'i-carbon-moon']"
            />
          </NFloatButton>
        </template>
        {{ isDark ? "亮色模式" : "暗色模式" }}
      </NTooltip>
    </template>
  </NFloatButton>

  <AiSettingsDialog v-model:show="showAiSettings" />
</template>
