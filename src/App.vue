<script setup lang="ts">
import { ref, watch, onMounted, onBeforeUnmount, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useReposStore } from "./stores/repos";
import { useRepoStateStore } from "./stores/repoState";
import { api } from "./api/tauri";
import type { RepoSnapshot } from "./types/git";
import RepoSwitcher from "./components/RepoSwitcher.vue";
import TitleBar from "./components/TitleBar.vue";
import StatusBar from "./components/StatusBar.vue";
import BranchesPanel from "./components/BranchesPanel.vue";
import CommitPanel from "./components/CommitPanel.vue";
import CommitDetailPanel from "./components/CommitDetailPanel.vue";
import LogPanel from "./components/LogPanel.vue";
import InProgressBanner from "./components/InProgressBanner.vue";
import ToastTray from "./components/ToastTray.vue";
import RebaseTodoDialog from "./components/RebaseTodoDialog.vue";
import CommitMessageDialog from "./components/CommitMessageDialog.vue";
import ResetDialog from "./components/ResetDialog.vue";
import BranchCreateDialog from "./components/BranchCreateDialog.vue";
import BranchDeleteDialog from "./components/BranchDeleteDialog.vue";
import ConfirmDialog from "./components/ConfirmDialog.vue";
import UpdateBanner from "./components/UpdateBanner.vue";
import { checkForUpdates } from "./lib/updater";

const repos = useReposStore();
const state = useRepoStateStore();

const SIDE_KEY = "pluck:sideWidth";
const MIN_W = 180, MAX_W = 600;
const sideWidth = ref<number>(loadWidth());
function loadWidth(): number {
  const n = Number(localStorage.getItem(SIDE_KEY));
  return Number.isFinite(n) && n >= MIN_W && n <= MAX_W ? n : 260;
}
watch(sideWidth, v => localStorage.setItem(SIDE_KEY, String(v)));
const gridCols = computed(() => `${sideWidth.value}px 6px 1fr`);

let dragStartX = 0; let dragStartW = 0;
function onDragMove(e: MouseEvent) {
  const next = dragStartW + (e.clientX - dragStartX);
  sideWidth.value = Math.max(MIN_W, Math.min(MAX_W, next));
}
function onDragEnd() {
  document.removeEventListener("mousemove", onDragMove);
  document.removeEventListener("mouseup", onDragEnd);
  document.body.style.cursor = "";
  document.body.style.userSelect = "";
}
function startDrag(e: MouseEvent) {
  dragStartX = e.clientX;
  dragStartW = sideWidth.value;
  document.addEventListener("mousemove", onDragMove);
  document.addEventListener("mouseup", onDragEnd);
  document.body.style.cursor = "col-resize";
  document.body.style.userSelect = "none";
  e.preventDefault();
}

// 右列上下分割：上半 commit/detail，下半 log。
// 用 ratio 持久化而不是 px：窗口 resize 时按比例还原，避免重启后比例失真。
const TOP_KEY = "pluck:rightTopRatio";
const MIN_RATIO = 0.15, MAX_RATIO = 0.85;
const rightTopRatio = ref<number>(loadRatio());
function loadRatio(): number {
  const n = Number(localStorage.getItem(TOP_KEY));
  return Number.isFinite(n) && n >= MIN_RATIO && n <= MAX_RATIO ? n : 0.5;
}
watch(rightTopRatio, v => localStorage.setItem(TOP_KEY, String(v)));
const rightCol = ref<HTMLElement | null>(null);
const rightRows = computed(() => `${(rightTopRatio.value * 100).toFixed(3)}% 6px 1fr`);

let dragStartY = 0; let dragStartRatio = 0; let dragColHeight = 0;
function onDragMoveV(e: MouseEvent) {
  if (!dragColHeight) return;
  const dyRatio = (e.clientY - dragStartY) / dragColHeight;
  const next = dragStartRatio + dyRatio;
  rightTopRatio.value = Math.max(MIN_RATIO, Math.min(MAX_RATIO, next));
}
function onDragEndV() {
  document.removeEventListener("mousemove", onDragMoveV);
  document.removeEventListener("mouseup", onDragEndV);
  document.body.style.cursor = "";
  document.body.style.userSelect = "";
}
function startDragV(e: MouseEvent) {
  dragColHeight = rightCol.value?.clientHeight ?? 0;
  if (!dragColHeight) return;
  dragStartY = e.clientY;
  dragStartRatio = rightTopRatio.value;
  document.addEventListener("mousemove", onDragMoveV);
  document.addEventListener("mouseup", onDragEndV);
  document.body.style.cursor = "row-resize";
  document.body.style.userSelect = "none";
  e.preventDefault();
}

watch(() => repos.activeId, async id => {
  if (!id) return;
  const meta = repos.all.find(r => r.id === id);
  if (meta) {
    try { await api.repoAdd(meta.path); }
    catch (e: any) { state.pushToast("error", e?.data?.friendly ?? String(e)); return; }
  }
  state.open(id);
}, { immediate: true });

function onFocus() { if (repos.activeId) state.refresh(repos.activeId); }
function isMeta(e: KeyboardEvent) { return e.metaKey || e.ctrlKey; }
async function onKey(e: KeyboardEvent) {
  if (e.key === "Escape" && state.selectedCommit) {
    const tag = (document.activeElement as HTMLElement | null)?.tagName;
    if (tag !== "INPUT" && tag !== "TEXTAREA") {
      e.preventDefault();
      state.clearSelectedCommit();
      return;
    }
  }
  if (!repos.activeId) return;
  if (isMeta(e) && e.key.toLowerCase() === "k" && !e.shiftKey) {
    e.preventDefault();
    (document.querySelector("textarea") as HTMLTextAreaElement | null)?.focus();
  } else if (isMeta(e) && e.shiftKey && e.key.toLowerCase() === "k") {
    e.preventDefault();
    try {
      state.snapshot = await invoke<RepoSnapshot>("push_branch", { id: repos.activeId, forceWithLease: false });
      state.pushToast("info", "Push successful");
    } catch (err: any) { state.pushToast("error", err?.data?.friendly ?? String(err)); }
  } else if (isMeta(e) && e.key.toLowerCase() === "t") {
    e.preventDefault();
    try {
      state.snapshot = await invoke<RepoSnapshot>("fetch", { id: repos.activeId });
      state.pushToast("info", "Fetch successful");
    } catch (err: any) { state.pushToast("error", err?.data?.friendly ?? String(err)); }
  } else if (isMeta(e) && e.key.toLowerCase() === "r") {
    e.preventDefault();
    state.refresh(repos.activeId);
  }
}
const UPDATE_RECHECK_MS = 6 * 60 * 60 * 1000;
let updateTimer: number | null = null;
onMounted(() => {
  window.addEventListener("focus", onFocus);
  window.addEventListener("keydown", onKey);
  checkForUpdates(false);
  updateTimer = window.setInterval(() => { checkForUpdates(false); }, UPDATE_RECHECK_MS);
});
onBeforeUnmount(() => {
  window.removeEventListener("focus", onFocus);
  window.removeEventListener("keydown", onKey);
  if (updateTimer !== null) { window.clearInterval(updateTimer); updateTimer = null; }
});
</script>

<template>
  <div class="flex h-full" style="background: var(--bg); color: var(--fg)">
    <RepoSwitcher />
    <div class="flex-1 flex flex-col min-w-0">
      <UpdateBanner />
      <TitleBar />
      <InProgressBanner v-if="state.snapshot?.inProgress" />
      <div class="flex-1 grid p-3 overflow-hidden"
           :style="{ gridTemplateColumns: gridCols }">
        <div class="gl-surface rounded-lg overflow-auto"
             style="border: 1px solid var(--border)">
          <BranchesPanel />
        </div>
        <div class="cursor-col-resize gl-splitter flex justify-center"
             @mousedown="startDrag"
             @dblclick="sideWidth = 260"
             title="Drag to resize · double-click to reset">
          <div class="gl-splitter-line" />
        </div>
        <div ref="rightCol" class="grid min-h-0 min-w-0"
             :style="{ gridTemplateRows: rightRows }">
          <div class="gl-surface rounded-lg overflow-auto min-h-0" style="border: 1px solid var(--border)">
            <CommitDetailPanel v-if="state.selectedCommit" />
            <CommitPanel v-else />
          </div>
          <div class="cursor-row-resize gl-splitter gl-splitter--h flex items-center"
               @mousedown="startDragV"
               @dblclick="rightTopRatio = 0.5"
               title="Drag to resize · double-click to reset">
            <div class="gl-splitter-line" />
          </div>
          <div class="gl-surface rounded-lg overflow-auto min-h-0" style="border: 1px solid var(--border)">
            <LogPanel />
          </div>
        </div>
      </div>
      <StatusBar />
    </div>
    <ToastTray />
    <RebaseTodoDialog />
    <CommitMessageDialog />
    <ResetDialog />
    <BranchCreateDialog />
    <BranchDeleteDialog />
    <ConfirmDialog />
  </div>
</template>
