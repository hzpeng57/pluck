<script setup lang="ts">
import { ref } from "vue";
import { useReposStore } from "../stores/repos";
import { useRepoStateStore } from "../stores/repoState";
import { invoke } from "@tauri-apps/api/core";
import type { RepoSnapshot } from "../types/git";

const repos = useReposStore();
const state = useRepoStateStore();
const showPushMenu = ref(false);

async function push(force: boolean) {
  if (!repos.activeId) return;
  showPushMenu.value = false;
  try { state.snapshot = await invoke<RepoSnapshot>("push_branch", { id: repos.activeId, forceWithLease: force }); }
  catch (e: any) { state.lastError = e?.data?.friendly ?? String(e); }
}

async function fetch() {
  if (!repos.activeId) return;
  try { state.snapshot = await invoke<RepoSnapshot>("fetch", { id: repos.activeId }); }
  catch (e: any) { state.lastError = e?.data?.friendly ?? String(e); }
}
</script>
<template>
  <div class="flex items-center px-3 py-1.5 border-b border-neutral-200 dark:border-neutral-800 gap-2">
    <div class="font-semibold truncate">{{ repos.active?.name ?? "No repo" }}</div>
    <div class="opacity-60 text-xs">{{ state.snapshot?.head.branch ?? "(no head)" }}</div>
    <div class="flex-1" />
    <button class="px-2 py-0.5 rounded hover:bg-neutral-200 dark:hover:bg-neutral-800" @click="fetch">Fetch</button>
    <div class="relative">
      <button class="px-2 py-0.5 rounded hover:bg-neutral-200 dark:hover:bg-neutral-800" @click="showPushMenu = !showPushMenu">Push ▾</button>
      <div v-if="showPushMenu" class="absolute right-0 mt-1 bg-white dark:bg-neutral-800 border rounded shadow z-50 min-w-48">
        <button class="block w-full text-left px-3 py-1 hover:bg-neutral-100 dark:hover:bg-neutral-700" @click="push(false)">Push</button>
        <button class="block w-full text-left px-3 py-1 hover:bg-neutral-100 dark:hover:bg-neutral-700" @click="push(true)">Push (force-with-lease)</button>
      </div>
    </div>
    <button class="px-2 py-0.5 rounded hover:bg-neutral-200 dark:hover:bg-neutral-800" disabled>Pull --rebase</button>
  </div>
</template>
