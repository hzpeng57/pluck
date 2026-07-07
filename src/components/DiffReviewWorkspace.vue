<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from "vue";
import { useReposStore } from "../stores/repos";
import { useRepoStateStore } from "../stores/repoState";
import CommitPanel from "./CommitPanel.vue";
import CommitDetailPanel from "./CommitDetailPanel.vue";
import DiffViewer from "./DiffViewer.vue";

const repos = useReposStore();
const state = useRepoStateStore();
const sourceIsCommit = computed(() => state.diffTarget?.kind === "commit");

const SOURCE_KEY = "pluck:diffSourceWidth";
const MIN_SOURCE_W = 280;
const MAX_SOURCE_W = 560;
const DEFAULT_SOURCE_W = 360;
const sourceWidth = ref(loadSourceWidth());
const gridCols = computed(() => `${sourceWidth.value}px 6px minmax(560px, 1fr)`);

function loadSourceWidth() {
  const n = Number(localStorage.getItem(SOURCE_KEY));
  return Number.isFinite(n) && n >= MIN_SOURCE_W && n <= MAX_SOURCE_W ? n : DEFAULT_SOURCE_W;
}
watch(sourceWidth, v => localStorage.setItem(SOURCE_KEY, String(v)));

let dragStartX = 0;
let dragStartW = 0;
function onSourceDragMove(e: MouseEvent) {
  const next = dragStartW + (e.clientX - dragStartX);
  sourceWidth.value = Math.max(MIN_SOURCE_W, Math.min(MAX_SOURCE_W, next));
}
function onSourceDragEnd() {
  document.removeEventListener("mousemove", onSourceDragMove);
  document.removeEventListener("mouseup", onSourceDragEnd);
  document.body.style.cursor = "";
  document.body.style.userSelect = "";
}
function startSourceDrag(e: MouseEvent) {
  dragStartX = e.clientX;
  dragStartW = sourceWidth.value;
  document.addEventListener("mousemove", onSourceDragMove);
  document.addEventListener("mouseup", onSourceDragEnd);
  document.body.style.cursor = "col-resize";
  document.body.style.userSelect = "none";
  e.preventDefault();
}
onBeforeUnmount(onSourceDragEnd);

async function rollback() {
  if (!repos.activeId || state.diffTarget?.kind !== "workingTree") return;
  const repoId = repos.activeId;
  const target = state.diffTarget;
  const destructive = target.status === "untracked" || target.status === "added" || target.status === "renamed";
  const ok = await state.confirmAction({
    title: "Rollback File",
    message: destructive
      ? `Rollback will remove "${target.path}" from your working tree.`
      : `Rollback will restore "${target.path}" to HEAD.`,
    confirmLabel: "Rollback",
    tone: destructive ? "danger" : "warning",
    confirmText: destructive ? target.path : undefined,
  });
  if (!ok) return;
  const current = state.diffTarget;
  if (
    !repos.activeId ||
    repos.activeId !== repoId ||
    current?.kind !== "workingTree" ||
    current.path !== target.path ||
    current.oldPath !== target.oldPath ||
    current.status !== target.status
  ) return;
  try {
    await state.rollbackCurrentWorkingFile(repoId);
    if (state.diffError) {
      state.pushToast("error", state.diffError);
      return;
    }
    state.pushToast("info", `Rolled back ${target.path}`);
  } catch (e: any) {
    state.pushToast("error", e?.data?.friendly ?? e?.message ?? String(e));
  }
}
</script>

<template>
  <div class="h-full min-h-0 min-w-0 grid" :style="{ gridTemplateColumns: gridCols }">
    <aside class="min-h-0 overflow-hidden" style="background: var(--panel)">
      <CommitDetailPanel v-if="sourceIsCommit" review-mode />
      <CommitPanel v-else review-mode />
    </aside>
    <div class="cursor-col-resize gl-splitter flex justify-center"
         @mousedown="startSourceDrag"
         @dblclick="sourceWidth = DEFAULT_SOURCE_W"
         title="Drag to resize file list · double-click to reset">
      <div class="gl-splitter-line" />
    </div>
    <DiffViewer @back="state.closeReviewMode()" @rollback="rollback" />
  </div>
</template>
