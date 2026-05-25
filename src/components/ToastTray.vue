<script setup lang="ts">
import { useRepoStateStore } from "../stores/repoState";
const state = useRepoStateStore();
</script>
<template>
  <div class="fixed bottom-4 right-4 flex flex-col gap-2 z-50">
    <transition-group name="toast">
      <div v-for="t in state.toasts" :key="t.id"
           class="flex items-start gap-2.5 px-3.5 py-2.5 rounded-lg shadow-2xl text-[12.5px] max-w-md"
           :style="{
             background: 'var(--raised)',
             border: '1px solid ' + (t.level === 'error' ? 'rgba(248,113,113,0.4)' : 'var(--border)'),
             color: 'var(--fg)',
             boxShadow: '0 10px 32px rgba(0,0,0,0.45), 0 1px 0 rgba(255,255,255,0.04) inset'
           }">
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
