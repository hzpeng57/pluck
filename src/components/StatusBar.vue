<script setup lang="ts">
import { computed } from "vue";
import { useRepoStateStore } from "../stores/repoState";
const state = useRepoStateStore();
const counts = computed(() => {
  const s = state.snapshot; if (!s) return { dirty: 0, ahead: 0, behind: 0 };
  return { dirty: s.files.length, ahead: s.remoteStatus.ahead, behind: s.remoteStatus.behind };
});
</script>
<template>
  <div class="text-xs px-3 py-1 border-t border-neutral-200 dark:border-neutral-800 flex gap-3 opacity-70">
    <span>✏ {{ counts.dirty }}</span>
    <span>↑ {{ counts.ahead }}</span>
    <span>↓ {{ counts.behind }}</span>
    <span v-if="state.loading">refreshing…</span>
  </div>
</template>
