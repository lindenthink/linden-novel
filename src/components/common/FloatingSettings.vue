<script setup lang="ts">
import { ref } from "vue";
import { NFloatButton, NTooltip } from "naive-ui";
import { useTheme } from "../../composables/useTheme";
import { useUpdater } from "../../composables/useUpdater";
import AiSettingsDialog from "../ai/AiSettingsDialog.vue";
import UpdateDialog from "./UpdateDialog.vue";

const { toggle, isDark } = useTheme();
const { checkForUpdates, showDialog } = useUpdater();

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

async function handleCheckUpdate() {
  showMenu.value = false;
  // 手动触发：不走节流，弹出 dialog 显示「检查中」→「available」或「not-available」
  await checkForUpdates(false);
}
</script>

<template>
  <NFloatButton
    v-model:show-menu="showMenu"
    menu-trigger="click"
    :bottom="80"
    :right="40"
    shape="circle"
    type="primary"
  >
    <span class="i-carbon-settings text-xl" />
    <template #menu>
       <NTooltip trigger="hover" placement="left">
        <template #trigger>
          <NFloatButton shape="circle" @click="handleCheckUpdate" type="primary">
            <span class="i-carbon-update-now text-lg" />
          </NFloatButton>
        </template>
        检查更新
      </NTooltip>
      <NTooltip trigger="hover" placement="left">
        <template #trigger>
          <NFloatButton shape="circle" @click="handleAiSettings" type="primary">
            <span class="i-carbon-ai text-lg" />
          </NFloatButton>
        </template>
        AI 设置
      </NTooltip>
      <NTooltip trigger="hover" placement="left">
        <template #trigger>
          <NFloatButton shape="circle" @click="handleThemeToggle" type="primary">
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
  <UpdateDialog v-model:show="showDialog" />
</template>
