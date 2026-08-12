<script setup lang="ts">
import { ref } from "vue";
import { NFloatButton, NTooltip, useMessage } from "naive-ui";
import { useTheme } from "../../composables/useTheme";
import { useEditorSettings } from "../../composables/useEditorSettings";
import AiSettingsDialog from "../ai/AiSettingsDialog.vue";

const { toggle, isDark } = useTheme();
const { autoSaveEnabled, setAutoSave } = useEditorSettings();
const showAiSettings = ref(false);
const showMenu = ref(false);
const message = useMessage();

function handleAiSettings() {
  showMenu.value = false;
  showAiSettings.value = true;
}

function handleThemeToggle() {
  showMenu.value = false;
  toggle();
}

async function handleAutoSaveToggle() {
  const next = !autoSaveEnabled.value;
  await setAutoSave(next);
  message.success(next ? "已启用自动保存" : "已关闭自动保存");
}
</script>

<template>
  <NFloatButton
    v-model:show-menu="showMenu"
    menu-trigger="click"
    :bottom="40"
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
      <NTooltip trigger="hover" placement="left">
        <template #trigger>
          <NFloatButton shape="circle" @click="handleAutoSaveToggle">
            <span
              :class="[autoSaveEnabled ? 'i-carbon-save' : 'i-carbon-save text-gray-400']"
            />
          </NFloatButton>
        </template>
        {{ autoSaveEnabled ? "自动保存：开" : "自动保存：关" }}
      </NTooltip>
    </template>
  </NFloatButton>

  <AiSettingsDialog v-model:show="showAiSettings" />
</template>
