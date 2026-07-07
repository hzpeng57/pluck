<script setup lang="ts">
import { useRepoStateStore } from "../stores/repoState";
const state = useRepoStateStore();
</script>
<template>
  <div class="fixed bottom-4 right-4 flex flex-col gap-2 z-[60]">
    <transition-group name="toast">
      <div v-for="t in state.toasts" :key="t.id"
           class="gl-toast flex items-start gap-2.5 px-3.5 py-2.5 rounded-lg text-[13px] max-w-md"
           :style="{ borderColor: t.level === 'error' ? 'var(--danger-ring)' : 'var(--border)' }">
        <span class="mt-0.5 w-1.5 h-1.5 rounded-full shrink-0"
              :style="{ background: t.level === 'error' ? 'var(--danger)' : 'var(--info)' }" />
        <span class="break-words whitespace-pre-wrap">{{ t.msg }}</span>
      </div>
    </transition-group>
  </div>
</template>

<style scoped>
.toast-enter-active, .toast-leave-active { transition: all 200ms ease; }
.toast-enter-from { opacity: 0; transform: translateX(20px); }
.toast-leave-to { opacity: 0; transform: translateX(20px); }
</style>
