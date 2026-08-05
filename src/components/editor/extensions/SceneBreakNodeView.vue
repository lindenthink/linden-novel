<script setup lang="ts">
import { NodeViewWrapper, type NodeViewProps } from "@tiptap/vue-3";

defineProps<NodeViewProps>();
</script>

<template>
  <NodeViewWrapper class="scene-break-wrapper">
    <div class="scene-break">
      <span class="scene-break-dots">* * *</span>
    </div>
  </NodeViewWrapper>
</template>

<style scoped>
.scene-break-wrapper {
  margin: 1.5rem 0;
}

.scene-break {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0.5rem 0;
}

.scene-break-dots {
  color: #9ca3af;
  font-size: 1.25rem;
  letter-spacing: 0.5rem;
  user-select: none;
}

:root.dark .scene-break-dots {
  color: #6b7280;
}
</style>
