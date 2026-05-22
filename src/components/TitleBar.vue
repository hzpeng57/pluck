<script setup lang="ts">
import { useReposStore } from "../stores/repos";
import { useRepoStateStore } from "../stores/repoState";
const repos = useReposStore(); const state = useRepoStateStore();
</script>
<template>
  <div class="flex items-center px-3 py-1.5 border-b border-neutral-200 dark:border-neutral-800 gap-2">
    <div class="font-semibold truncate">{{ repos.active?.name ?? "No repo" }}</div>
    <div class="opacity-60 text-xs">{{ state.snapshot?.head.branch ?? "(no head)" }}</div>
    <div class="flex-1" />
    <button class="px-2 py-0.5 rounded hover:bg-neutral-200 dark:hover:bg-neutral-800" disabled>Fetch</button>
    <button class="px-2 py-0.5 rounded hover:bg-neutral-200 dark:hover:bg-neutral-800" disabled>Push</button>
    <button class="px-2 py-0.5 rounded hover:bg-neutral-200 dark:hover:bg-neutral-800" disabled>Pull --rebase</button>
  </div>
</template>
