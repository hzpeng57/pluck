<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from "vue";
import { openPath, revealItemInDir } from "@tauri-apps/plugin-opener";
import { useReposStore } from "../stores/repos";
import { useRepoStateStore } from "../stores/repoState";
import CommitPanel from "./CommitPanel.vue";
import CommitDetailPanel from "./CommitDetailPanel.vue";
import DiffViewer from "./DiffViewer.vue";
import type { ChangedFile, DiffTarget, WorkingFile } from "../types/git";

const repos = useReposStore();
const state = useRepoStateStore();
const sourceIsCommit = computed(() => state.diffTarget?.kind === "commit");
type ReviewFile = WorkingFile | ChangedFile;

const reviewFiles = computed<ReviewFile[]>(() =>
  sourceIsCommit.value ? (state.selectedCommit?.files ?? []) : (state.snapshot?.files ?? [])
);

const currentIndex = computed(() => {
  const target = state.diffTarget;
  if (!target) return -1;
  return reviewFiles.value.findIndex(file =>
    file.path === target.path &&
    file.oldPath === target.oldPath &&
    file.status === target.status
  );
});

const canGoPrevious = computed(() => currentIndex.value > 0);
const canGoNext = computed(() => currentIndex.value >= 0 && currentIndex.value < reviewFiles.value.length - 1);
const canUseWorkingPath = computed(() => state.diffTarget?.kind === "workingTree" && !!repos.active);
const canOpenFile = computed(() =>
  canUseWorkingPath.value &&
  state.diffTarget?.status !== "deleted" &&
  state.diffTarget?.status !== "conflicted"
);
const canRevealFile = computed(() => canOpenFile.value);

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

function joinRepoPath(path: string, rel: string) {
  const cleanRoot = path.replace(/\/+$/, "");
  const cleanRel = rel.replace(/^\/+/, "").replace(/\/+$/, "");
  return `${cleanRoot}/${cleanRel}`;
}

function targetDisplayPath(target: DiffTarget | null) {
  if (!target) return "";
  return target.oldPath ? `${target.oldPath} -> ${target.path}` : target.path;
}

function targetFsPath(target: DiffTarget | null) {
  if (!target || !repos.active) return null;
  const rel = target.status === "deleted" && target.oldPath ? target.oldPath : target.path;
  return joinRepoPath(repos.active.path, rel);
}

async function openReviewFileAt(index: number) {
  if (!repos.activeId) return;
  const file = reviewFiles.value[index];
  if (!file) return;
  if (sourceIsCommit.value && state.selectedCommit) {
    await state.openCommitFileDiff(repos.activeId, state.selectedCommit, file as ChangedFile);
  } else {
    await state.openWorkingDiff(repos.activeId, file as WorkingFile);
  }
}

async function previousFile() {
  if (canGoPrevious.value) await openReviewFileAt(currentIndex.value - 1);
}

async function nextFile() {
  if (canGoNext.value) await openReviewFileAt(currentIndex.value + 1);
}

async function copyPath() {
  const path = targetDisplayPath(state.diffTarget);
  if (!path) return;
  try {
    await navigator.clipboard.writeText(path);
    state.pushToast("info", "Path copied");
  } catch (e: any) {
    state.pushToast("error", e?.message ?? String(e));
  }
}

async function revealFile() {
  const path = targetFsPath(state.diffTarget);
  if (!path || !canRevealFile.value) return;
  try {
    await revealItemInDir(path);
  } catch (e: any) {
    state.pushToast("error", e?.message ?? String(e));
  }
}

async function openFile() {
  const path = targetFsPath(state.diffTarget);
  if (!path || !canOpenFile.value) return;
  try {
    await openPath(path);
  } catch (e: any) {
    state.pushToast("error", e?.message ?? String(e));
  }
}

function pickNextAfterRollback(beforeIndex: number): WorkingFile | null {
  const files = state.snapshot?.files ?? [];
  if (files.length === 0) return null;
  const clamped = Math.min(beforeIndex >= 0 ? beforeIndex : 0, files.length - 1);
  return files[clamped] ?? files[files.length - 1] ?? null;
}

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
  const beforeIndex = currentIndex.value;
  try {
    await state.rollbackCurrentWorkingFile(repoId);
    if (state.diffError) {
      state.pushToast("error", state.diffError);
      return;
    }
    state.pushToast("info", `Rolled back ${target.path}`);
    const next = pickNextAfterRollback(beforeIndex);
    if (next && repos.activeId === repoId) {
      await state.openWorkingDiff(repoId, next);
    }
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
    <DiffViewer
      :can-go-previous="canGoPrevious"
      :can-go-next="canGoNext"
      :can-open-file="canOpenFile"
      :can-reveal-file="canRevealFile"
      @back="state.closeReviewMode()"
      @rollback="rollback"
      @previous-file="previousFile"
      @next-file="nextFile"
      @copy-path="copyPath"
      @open-file="openFile"
      @reveal-file="revealFile"
    />
  </div>
</template>
