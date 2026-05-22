<script setup lang="ts">
import { useRepoStateStore } from "../stores/repoState";
const state = useRepoStateStore();
</script>
<template>
  <div class="fixed bottom-4 right-4 flex flex-col gap-2 z-50">
    <div v-for="t in state.toasts" :key="t.id"
         :class="['px-3 py-2 rounded shadow text-sm max-w-md',
                  t.level === 'error' ? 'bg-red-600 text-white' : 'bg-neutral-800 text-white']">
      {{ t.msg }}
    </div>
  </div>
</template>
