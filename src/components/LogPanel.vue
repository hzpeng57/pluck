<script setup lang="ts">
import { computed } from "vue";
import { useRepoStateStore } from "../stores/repoState";
import { relativeTime } from "../lib/format";

const state = useRepoStateStore();
const log = computed(() => state.snapshot?.log ?? []);
</script>

<template>
  <div class="flex flex-col">
    <div class="px-2 py-1 text-xs uppercase opacity-50">Log: {{ state.selectedLogBranch ?? "(none)" }}</div>
    <ul class="overflow-auto">
      <li v-for="c in log" :key="c.hash" class="px-2 py-1 hover:bg-neutral-200 dark:hover:bg-neutral-800 flex items-baseline gap-2">
        <span class="font-mono text-xs opacity-60 w-16 shrink-0">{{ c.short }}</span>
        <span class="flex-1 truncate">{{ c.subject }}</span>
        <span class="text-xs opacity-50 shrink-0">{{ c.author }}</span>
        <span class="text-xs opacity-50 shrink-0">{{ relativeTime(c.dateUnix) }}</span>
      </li>
      <li v-if="log.length === 0" class="opacity-50 px-2 py-2">No commits.</li>
    </ul>
  </div>
</template>
