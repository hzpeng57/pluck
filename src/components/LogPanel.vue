<script setup lang="ts">
import { computed, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useRepoStateStore } from "../stores/repoState";
import { useReposStore } from "../stores/repos";
import { relativeTime } from "../lib/format";
import type { Commit, RepoSnapshot } from "../types/git";

const state = useRepoStateStore();
const repos = useReposStore();
const log = computed(() => state.snapshot?.log ?? []);

const menu = ref<{ x: number; y: number; commit: Commit } | null>(null);
function onContext(e: MouseEvent, c: Commit) { menu.value = { x: e.clientX, y: e.clientY, commit: c }; }
window.addEventListener("click", () => menu.value = null);

async function interactiveRebase() {
  if (!menu.value || !repos.activeId) return;
  const id = repos.activeId; const from = menu.value.commit.hash;
  menu.value = null;
  try { state.snapshot = await invoke<RepoSnapshot>("rebase_interactive_start", { id, fromCommit: from }); }
  catch (e: any) { state.lastError = e?.data?.friendly ?? String(e); }
}
</script>

<template>
  <div class="flex flex-col">
    <div class="px-2 py-1 text-xs uppercase opacity-50">Log: {{ state.selectedLogBranch ?? "(none)" }}</div>
    <ul class="overflow-auto">
      <li v-for="c in log" :key="c.hash"
          @contextmenu.prevent="onContext($event, c)"
          class="px-2 py-1 hover:bg-neutral-200 dark:hover:bg-neutral-800 flex items-baseline gap-2">
        <span class="font-mono text-xs opacity-60 w-16 shrink-0">{{ c.short }}</span>
        <span class="flex-1 truncate">{{ c.subject }}</span>
        <span class="text-xs opacity-50 shrink-0">{{ c.author }}</span>
        <span class="text-xs opacity-50 shrink-0">{{ relativeTime(c.dateUnix) }}</span>
      </li>
      <li v-if="log.length === 0" class="opacity-50 px-2 py-2">No commits.</li>
    </ul>
    <div v-if="menu" :style="{ top: menu.y + 'px', left: menu.x + 'px' }"
         class="fixed z-50 bg-white dark:bg-neutral-800 border rounded shadow text-sm min-w-56">
      <button class="block w-full text-left px-3 py-1 hover:bg-neutral-100 dark:hover:bg-neutral-700"
              @click="interactiveRebase">Interactively rebase from here</button>
    </div>
  </div>
</template>
