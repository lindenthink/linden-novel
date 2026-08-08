import { ref } from "vue";

/**
 * 编辑器 UI 共享状态
 * — showAIGenerationDialog 由顶部工具栏触发，编辑器内渲染对话框
 */
const showAIGenerationDialog = ref(false);

export function useEditorUI() {
  return { showAIGenerationDialog };
}
