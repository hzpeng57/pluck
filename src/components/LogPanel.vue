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

function onCommitClick(c: Commit) {
  if (!repos.activeId) return;
  state.selectCommit(repos.activeId, c.hash);
}

async function interactiveRebase() {
  if (!menu.value || !repos.activeId) return;
  const id = repos.activeId; const from = menu.value.commit.hash;
  menu.value = null;
  try { state.snapshot = await invoke<RepoSnapshot>("rebase_interactive_start", { id, fromCommit: from }); }
  catch (e: any) { state.pushToast("error", e?.data?.friendly ?? String(e)); }
}

function authorInitial(name: string) {
  return name.trim().split(/\s+/).map(w => w[0]).slice(0, 2).join("").toUpperCase() || "·";
}
function authorColor(name: string) {
  let h = 0; for (const c of name) h = (h * 31 + c.charCodeAt(0)) >>> 0;
  const palette = ["#6366F1", "#EC4899", "#10B981", "#F59E0B", "#06B6D4", "#8B5CF6", "#F43F5E", "#84CC16"];
  return palette[h % palette.length];
}
</script>

<template>
  <div class="flex flex-col h-full">
    <div class="flex items-center gap-2 px-3 pt-3 pb-2">
      <span class="gl-section-title">History</span>
      <span class="gl-mono text-[11px] px-1.5 py-0.5 rounded"
            style="background: var(--accent-soft); color: var(--accent-2)">
        {{ state.selectedLogBranch ?? "—" }}
      </span>
      <div class="flex-1" />
      <span class="gl-chip">{{ log.length }}</span>
    </div>
    <ul class="flex-1 overflow-auto px-2 flex flex-col gap-0.5">
      <li v-for="c in log" :key="c.hash"
          @click="onCommitClick(c)"
          @contextmenu.prevent="onContext($event, c)"
          :title="c.subject"
          class="flex items-center gap-2.5 px-2 h-8 rounded-md cursor-pointer transition-colors"
          :class="{ 'gl-row-active': state.selectedCommit?.hash === c.hash }"
          @mouseover="(e: any) => { if (state.selectedCommit?.hash !== c.hash) e.currentTarget.style.background = 'var(--hover)' }"
          @mouseleave="(e: any) => { if (state.selectedCommit?.hash !== c.hash) e.currentTarget.style.background = '' }">
        <span class="gl-mono text-[10.5px] px-1.5 py-0.5 rounded shrink-0"
              style="background: var(--hover); color: var(--fg-3)">{{ c.short }}</span>
        <span class="inline-flex items-center justify-center w-5 h-5 rounded-full text-[9px] font-semibold shrink-0"
              :style="{ background: authorColor(c.author), color: '#fff' }"
              :title="c.author">{{ authorInitial(c.author) }}</span>
        <span class="flex-1 truncate text-[12.5px]" style="color: var(--fg)">{{ c.subject }}</span>
        <span class="text-[11px] shrink-0" style="color: var(--fg-3)">{{ relativeTime(c.dateUnix) }}</span>
      </li>
      <li v-if="log.length === 0"
          class="flex flex-col items-center justify-center gap-1 py-8 text-center"
          style="color: var(--fg-3)">
        <span class="text-2xl">∅</span>
        <span class="text-[12px]">No commits</span>
      </li>
    </ul>
    <div v-if="menu" :style="{ top: menu.y + 'px', left: menu.x + 'px' }" class="gl-menu">
      <button class="gl-menu-item" @click="interactiveRebase">Interactively rebase from here…</button>
    </div>
  </div>
</template>
