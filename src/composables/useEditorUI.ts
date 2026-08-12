import { ref } from "vue";

export type AIGenerationMode =
  | "continuation"
  | "expansion"
  | "rewrite"
  | "polish"
  | "outline";

/**
 * 编辑器 UI 共享状态
 * — showAIGenerationDialog 由顶部工具栏触发，编辑器内渲染对话框
 * — aiGenerationDefaultMode：打开对话框时默认选中的生成模式
 */
const showAIGenerationDialog = ref(false);
const aiGenerationDefaultMode = ref<AIGenerationMode>("continuation");

export function useEditorUI() {
  function openAIGeneration(mode: AIGenerationMode = "continuation") {
    aiGenerationDefaultMode.value = mode;
    showAIGenerationDialog.value = true;
  }

  return {
    showAIGenerationDialog,
    aiGenerationDefaultMode,
    openAIGeneration,
  };
}
