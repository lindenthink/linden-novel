import { ref } from "vue";

/**
 * 编辑器 UI 共享状态
 * — focusMode / typewriterMode 在编辑器和状态栏之间共享
 * — showAIGenerationDialog 由状态栏触发，编辑器内渲染对话框
 */
const focusMode = ref(false);
const typewriterMode = ref(false);
const showAIGenerationDialog = ref(false);

export function useEditorUI() {
  return { focusMode, typewriterMode, showAIGenerationDialog };
}
