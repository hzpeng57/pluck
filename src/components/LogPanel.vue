<script setup lang="ts">
import { computed, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useRepoStateStore } from "../stores/repoState";
import { useReposStore } from "../stores/repos";
import { api } from "../api/tauri";
import { relativeTime } from "../lib/format";
import type { Commit, RepoSnapshot } from "../types/git";

const state = useRepoStateStore();
const repos = useReposStore();
const log = computed(() => state.snapshot?.log ?? []);
const headBranch = computed(() => state.snapshot?.head.branch ?? null);
const onCurrentBranchLog = computed(() =>
  headBranch.value !== null && state.selectedLogBranch === headBranch.value
);

const menu = ref<{ x: number; y: number; commit: Commit } | null>(null);
window.addEventListener("click", () => menu.value = null);

function isSelected(hash: string) { return state.selectedHashes.has(hash); }

function onCommitClick(e: MouseEvent, c: Commit) {
  if (!repos.activeId) return;
  if (e.shiftKey) {
    state.selectRange(log.value, c.hash);
    return;
  }
  if (e.metaKey || e.ctrlKey) {
    state.toggleSelection(c.hash);
    return;
  }
  state.setSingleSelection(repos.activeId, c.hash);
}

function onContext(e: MouseEvent, c: Commit) {
  if (!repos.activeId) return;
  if (!isSelected(c.hash)) state.setSingleSelection(repos.activeId, c.hash);
  menu.value = { x: e.clientX, y: e.clientY, commit: c };
}

const selectedCount = computed(() => state.selectionCount);

// Selected commits in log order (oldest → newest replay order).
const selectedInOrder = computed<Commit[]>(() => {
  const set = state.selectedHashes;
  return log.value.filter(c => set.has(c.hash)).slice().reverse();
});

const singleSelected = computed<Commit | null>(() => {
  if (state.selectionCount !== 1) return null;
  return log.value.find(c => state.selectedHashes.has(c.hash)) ?? null;
});

// Edit Message enablement: single-select, non-merge, reachable from HEAD
// (we approximate "reachable from HEAD" as: viewing the current HEAD branch's log).
const canEditMessage = computed(() => {
  const c = singleSelected.value;
  if (!c) return false;
  if (c.parents.length !== 1) return false;
  return onCurrentBranchLog.value;
});

const editMessageMode = computed<"amend" | "reword" | null>(() => {
  const c = singleSelected.value;
  if (!c || !canEditMessage.value) return null;
  return log.value[0]?.hash === c.hash ? "amend" : "reword";
});

const canReset = computed(() => state.selectionCount === 1 && onCurrentBranchLog.value);

async function doCherryPick() {
  if (!repos.activeId || selectedCount.value === 0) return;
  const id = repos.activeId;
  const hashes = selectedInOrder.value.map(c => c.hash);
  menu.value = null;
  try { state.snapshot = await api.cherryPick(id, hashes); }
  catch (e: any) { state.pushToast("error", e?.data?.friendly ?? String(e)); }
}

async function doRevert() {
  if (!repos.activeId || selectedCount.value === 0) return;
  const id = repos.activeId;
  // Revert wants newest → oldest so the working tree stays sane.
  const hashes = selectedInOrder.value.map(c => c.hash).reverse();
  menu.value = null;
  try { state.snapshot = await api.revert(id, hashes); }
  catch (e: any) { state.pushToast("error", e?.data?.friendly ?? String(e)); }
}

function doEditMessage() {
  const c = singleSelected.value;
  const mode = editMessageMode.value;
  if (!c || !mode) return;
  const initial = c.body ? `${c.subject}\n\n${c.body}` : c.subject;
  menu.value = null;
  state.openEditMessageDialog(c.hash, initial, mode);
}

function doReset() {
  const c = singleSelected.value;
  if (!c) return;
  menu.value = null;
  state.openResetDialog(c.hash, c.short, c.subject);
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
      <span v-if="selectedCount > 1" class="gl-chip" style="background: var(--accent-soft); color: var(--accent-2)">
        {{ selectedCount }} selected
      </span>
      <span class="gl-chip">{{ log.length }}</span>
    </div>
    <ul class="flex-1 overflow-auto px-2 flex flex-col gap-0.5">
      <li v-for="c in log" :key="c.hash"
          @click="onCommitClick($event, c)"
          @contextmenu.prevent="onContext($event, c)"
          :title="c.subject"
          class="flex items-center gap-2.5 px-2 h-8 rounded-md cursor-pointer transition-colors"
          :class="{ 'gl-row-active': isSelected(c.hash) }"
          @mouseover="(e: any) => { if (!isSelected(c.hash)) e.currentTarget.style.background = 'var(--hover)' }"
          @mouseleave="(e: any) => { if (!isSelected(c.hash)) e.currentTarget.style.background = '' }">
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
      <button class="gl-menu-item" @click="doCherryPick">
        Cherry-Pick{{ selectedCount > 1 ? ` ${selectedCount} commits` : "" }}
      </button>
      <button class="gl-menu-item" @click="doRevert">
        Revert{{ selectedCount > 1 ? ` ${selectedCount} commits` : " Commit" }}
      </button>
      <button class="gl-menu-item"
              :disabled="!canEditMessage"
              :title="canEditMessage ? '' : 'Only single non-merge commits on the current branch can be edited'"
              @click="doEditMessage">
        Edit Commit Message…
      </button>
      <button class="gl-menu-item"
              :disabled="!canReset"
              :title="canReset ? '' : 'Select a single commit on the current branch'"
              @click="doReset">
        Reset Current Branch to Here…
      </button>
      <div class="gl-menu-sep" />
      <button class="gl-menu-item" @click="interactiveRebase">Interactively rebase from here…</button>
    </div>
  </div>
</template>
