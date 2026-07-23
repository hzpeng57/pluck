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
import DiffReviewWorkspace from "./components/DiffReviewWorkspace.vue";
import ConflictWorkspace from "./components/ConflictWorkspace.vue";
import LogPanel from "./components/LogPanel.vue";
import InProgressBanner from "./components/InProgressBanner.vue";
import ToastTray from "./components/ToastTray.vue";
import RebaseTodoDialog from "./components/RebaseTodoDialog.vue";
import CommitMessageDialog from "./components/CommitMessageDialog.vue";
import ResetDialog from "./components/ResetDialog.vue";
import BranchCreateDialog from "./components/BranchCreateDialog.vue";
import BranchRenameDialog from "./components/BranchRenameDialog.vue";
import BranchDeleteDialog from "./components/BranchDeleteDialog.vue";
import ConfirmDialog from "./components/ConfirmDialog.vue";
import UpdateBanner from "./components/UpdateBanner.vue";
import { checkForUpdates } from "./lib/updater";

const repos = useReposStore();
const state = useRepoStateStore();

const SIDE_KEY = "pluck:sideWidth";
const MIN_W = 220, MAX_W = 560;
const sideWidth = ref<number>(loadWidth());
function loadWidth(): number {
  const n = Number(localStorage.getItem(SIDE_KEY));
  return Number.isFinite(n) && n >= MIN_W && n <= MAX_W ? n : 292;
}
watch(sideWidth, v => localStorage.setItem(SIDE_KEY, String(v)));

const INSPECTOR_KEY = "pluck:inspectorWidth";
const MIN_INSPECTOR_W = 320, MAX_INSPECTOR_W = 560;
const inspectorWidth = ref<number>(loadInspectorWidth());
function loadInspectorWidth(): number {
  const n = Number(localStorage.getItem(INSPECTOR_KEY));
  return Number.isFinite(n) && n >= MIN_INSPECTOR_W && n <= MAX_INSPECTOR_W ? n : 390;
}
watch(inspectorWidth, v => localStorage.setItem(INSPECTOR_KEY, String(v)));
const reviewMode = computed(() => state.diffTarget !== null);
const conflictMode = computed(() => state.conflictWorkspaceOpen);
const gridCols = computed(() =>
  conflictMode.value
    ? "minmax(0, 1fr)"
    : reviewMode.value
    ? `${sideWidth.value}px 6px minmax(920px, 1fr)`
    : `${sideWidth.value}px 6px minmax(380px, 1fr) 6px ${inspectorWidth.value}px`
);

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

let inspectorDragStartX = 0; let inspectorDragStartW = 0;
function onInspectorDragMove(e: MouseEvent) {
  const next = inspectorDragStartW - (e.clientX - inspectorDragStartX);
  inspectorWidth.value = Math.max(MIN_INSPECTOR_W, Math.min(MAX_INSPECTOR_W, next));
}
function onInspectorDragEnd() {
  document.removeEventListener("mousemove", onInspectorDragMove);
  document.removeEventListener("mouseup", onInspectorDragEnd);
  document.body.style.cursor = "";
  document.body.style.userSelect = "";
}
function startInspectorDrag(e: MouseEvent) {
  inspectorDragStartX = e.clientX;
  inspectorDragStartW = inspectorWidth.value;
  document.addEventListener("mousemove", onInspectorDragMove);
  document.addEventListener("mouseup", onInspectorDragEnd);
  document.body.style.cursor = "col-resize";
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
  if (repos.activeId !== id) return;
  await state.open(id);
}, { immediate: true });

function onFocus() { if (repos.activeId) state.refresh(repos.activeId); }
function isMeta(e: KeyboardEvent) { return e.metaKey || e.ctrlKey; }
async function onKey(e: KeyboardEvent) {
  if (e.key === "Escape" && state.diffTarget) {
    const tag = (document.activeElement as HTMLElement | null)?.tagName;
    if (tag !== "INPUT" && tag !== "TEXTAREA") {
      e.preventDefault();
      state.closeReviewMode();
      return;
    }
  }
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
  <div class="gl-app-shell flex h-full">
    <RepoSwitcher />
    <div class="flex-1 flex flex-col min-w-0">
      <UpdateBanner />
      <TitleBar />
      <InProgressBanner v-if="state.snapshot?.inProgress" />
      <div class="flex-1 grid gap-0 px-3 py-3 overflow-hidden"
           :style="{ gridTemplateColumns: gridCols }">
        <template v-if="conflictMode">
          <div class="gl-panel overflow-hidden min-h-0 min-w-0">
            <ConflictWorkspace />
          </div>
        </template>
        <template v-else>
          <div class="gl-panel overflow-auto min-h-0">
            <BranchesPanel />
          </div>
          <div class="cursor-col-resize gl-splitter flex justify-center"
               @mousedown="startDrag"
               @dblclick="sideWidth = 292"
               title="Drag to resize · double-click to reset">
            <div class="gl-splitter-line" />
          </div>
          <template v-if="reviewMode">
            <div class="gl-panel overflow-hidden min-h-0 min-w-0">
              <DiffReviewWorkspace />
            </div>
          </template>
          <template v-else>
            <div class="gl-panel overflow-auto min-h-0 min-w-0">
              <LogPanel />
            </div>
            <div class="cursor-col-resize gl-splitter flex justify-center"
                 @mousedown="startInspectorDrag"
                 @dblclick="inspectorWidth = 390"
                 title="Drag to resize inspector · double-click to reset">
              <div class="gl-splitter-line" />
            </div>
            <div class="gl-panel overflow-auto min-h-0 min-w-0">
              <CommitDetailPanel v-if="state.selectedCommit" />
              <CommitPanel v-else />
            </div>
          </template>
        </template>
      </div>
      <StatusBar />
    </div>
    <ToastTray />
    <RebaseTodoDialog />
    <CommitMessageDialog />
    <ResetDialog />
    <BranchCreateDialog />
    <BranchRenameDialog />
    <BranchDeleteDialog />
    <ConfirmDialog />
  </div>
</template>
