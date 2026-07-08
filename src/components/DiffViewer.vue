<script setup lang="ts">
import { computed, ref, watch } from "vue";
import {
  ArrowDown,
  ArrowLeft,
  ArrowUp,
  Columns2,
  Copy,
  ExternalLink,
  FolderOpen,
  ListFilter,
  PanelTopBottomDashed,
  RotateCcw,
  WrapText,
} from "lucide-vue-next";
import { useRepoStateStore } from "../stores/repoState";
import { useReposStore } from "../stores/repos";
import type { DiffLine, DiffTarget, FileDiff } from "../types/git";
import { toSplitDiffHunks, type SplitDiffCell } from "../lib/diffLayout";

const emit = defineEmits<{
  back: [];
  rollback: [];
  previousFile: [];
  nextFile: [];
  copyPath: [];
  openFile: [];
  revealFile: [];
}>();

const props = withDefaults(defineProps<{
  canGoPrevious?: boolean;
  canGoNext?: boolean;
  canOpenFile?: boolean;
  canRevealFile?: boolean;
}>(), {
  canGoPrevious: false,
  canGoNext: false,
  canOpenFile: false,
  canRevealFile: false,
});

const state = useRepoStateStore();
const repos = useReposStore();
const wrap = ref(false);
type DiffViewMode = "unified" | "split";
const MODE_KEY = "pluck:diffViewMode";
const mode = ref<DiffViewMode>(loadMode());

function loadMode(): DiffViewMode {
  return localStorage.getItem(MODE_KEY) === "split" ? "split" : "unified";
}

watch(mode, value => localStorage.setItem(MODE_KEY, value));

const diff = computed(() => state.selectedDiff);
const splitHunks = computed(() => toSplitDiffHunks(diff.value));
const showSplit = computed(() => mode.value === "split" && !!diff.value && !diff.value.binary && diff.value.hunks.length > 0);
const target = computed(() => state.diffTarget);
const isWorkingTreeTarget = computed(() => target.value?.kind === "workingTree");
const isConflictedDiff = computed(() => (diff.value?.status ?? target.value?.status) === "conflicted");
const canRollback = computed(() => isWorkingTreeTarget.value && !isConflictedDiff.value);
const displayStatus = computed(() => diff.value?.status ?? target.value?.status ?? null);
const displayPath = computed(() => formatPath(diff.value ?? target.value));

function rowClass(line: DiffLine) {
  return {
    "is-added": line.kind === "added",
    "is-deleted": line.kind === "deleted",
  };
}

function mark(line: DiffLine) {
  if (line.kind === "added") return "+";
  if (line.kind === "deleted") return "-";
  if (line.kind === "noNewline") return "\\";
  return "";
}

function cellClass(cell: SplitDiffCell) {
  return {
    "is-added": cell.kind === "added",
    "is-deleted": cell.kind === "deleted",
    "is-empty": cell.kind === "empty",
    "is-notice": cell.kind === "notice",
  };
}

function formatPath(file: FileDiff | DiffTarget | null) {
  if (!file) return "";
  return file.oldPath ? `${file.oldPath} -> ${file.path}` : file.path;
}
</script>

<template>
  <section class="gl-diff-shell">
    <div class="gl-diff-toolbar">
      <button class="gl-command-btn h-7 px-2" title="Back to History" @click="emit('back')">
        <ArrowLeft :size="13" />
        Back
      </button>

      <template v-if="displayStatus">
        <span class="gl-badge">{{ displayStatus }}</span>
        <span class="min-w-0 flex-1 truncate text-[12.5px] gl-selectable" :title="displayPath" style="color: var(--fg-2)">
          {{ displayPath }}
        </span>
        <template v-if="diff">
          <span class="gl-badge" style="color: var(--success)">{{ diff.tooLarge ? "partial " : "" }}+{{ diff.additions }}</span>
          <span class="gl-badge" style="color: var(--danger)">{{ diff.tooLarge ? "partial " : "" }}-{{ diff.deletions }}</span>
          <span v-if="diff.tooLarge" class="gl-badge" style="color: var(--warning)">truncated</span>
        </template>
      </template>
      <div v-else class="flex-1" />

      <div class="gl-diff-action-group shrink-0">
        <button class="gl-icon-btn h-7 w-7"
                :disabled="!props.canGoPrevious"
                title="Previous changed file"
                @click="emit('previousFile')">
          <ArrowUp :size="13" />
        </button>
        <button class="gl-icon-btn h-7 w-7"
                :disabled="!props.canGoNext"
                title="Next changed file"
                @click="emit('nextFile')">
          <ArrowDown :size="13" />
        </button>
        <button class="gl-icon-btn h-7 w-7"
                :disabled="!displayPath"
                title="Copy path"
                @click="emit('copyPath')">
          <Copy :size="13" />
        </button>
        <button class="gl-icon-btn h-7 w-7"
                :disabled="!props.canRevealFile"
                title="Reveal in Finder"
                @click="emit('revealFile')">
          <FolderOpen :size="13" />
        </button>
        <button class="gl-icon-btn h-7 w-7"
                :disabled="!props.canOpenFile"
                title="Open file"
                @click="emit('openFile')">
          <ExternalLink :size="13" />
        </button>
      </div>

      <div class="gl-segmented shrink-0" title="Diff layout">
        <button class="gl-segmented-btn"
                :class="{ 'is-active': mode === 'unified' }"
                :aria-pressed="mode === 'unified'"
                title="Unified diff"
                @click="mode = 'unified'">
          <PanelTopBottomDashed :size="13" />
        </button>
        <button class="gl-segmented-btn"
                :class="{ 'is-active': mode === 'split' }"
                :aria-pressed="mode === 'split'"
                title="Side-by-side diff"
                @click="mode = 'split'">
          <Columns2 :size="13" />
        </button>
      </div>
      <button class="gl-command-btn h-7 px-2"
              :class="{ 'gl-btn-primary': wrap }"
              :aria-pressed="wrap"
              title="Toggle line wrap"
              @click="wrap = !wrap">
        <WrapText :size="13" />
        Wrap
      </button>
      <button class="gl-command-btn h-7 px-2"
              :class="{ 'gl-btn-primary': state.diffIgnoreWhitespace }"
              :aria-pressed="state.diffIgnoreWhitespace"
              title="Ignore whitespace changes"
              @click="repos.activeId && state.setDiffIgnoreWhitespace(repos.activeId, !state.diffIgnoreWhitespace)">
        <ListFilter :size="13" />
        Whitespace
      </button>
      <button v-if="isWorkingTreeTarget"
              class="gl-command-btn gl-btn-danger h-7 px-2"
              :disabled="!canRollback"
              :title="isConflictedDiff ? 'Resolve conflicts before rollback' : 'Rollback this file'"
              @click="emit('rollback')">
        <RotateCcw :size="13" />
        Rollback
      </button>
    </div>

    <div v-if="state.loadingDiff" class="gl-empty flex-1">
      <span class="gl-spinner" />
      <span>Loading diff...</span>
    </div>
    <div v-else-if="state.diffError" class="gl-empty flex-1" style="color: var(--danger)">
      <span class="text-[13px]">Diff failed</span>
      <span class="text-[12px] gl-selectable">{{ state.diffError }}</span>
    </div>
    <div v-else-if="!diff" class="gl-empty flex-1">
      <span class="text-[13px]">Select a file to review changes</span>
    </div>
    <div v-else-if="diff.binary" class="gl-empty flex-1">
      <span class="text-[13px]">Binary file diff is not available</span>
      <span class="text-[12.5px] gl-selectable">{{ diff.path }}</span>
    </div>
    <div v-else-if="diff.hunks.length === 0" class="gl-empty flex-1">
      <span class="text-[13px]">No textual changes</span>
      <span class="text-[12.5px] gl-selectable">{{ diff.path }}</span>
    </div>
    <div v-else class="gl-diff-scroll" :class="{ 'is-wrap': wrap, 'is-split': showSplit }">
      <table v-if="!showSplit" class="gl-diff-table">
        <tbody v-for="(hunk, hunkIndex) in diff.hunks" :key="`${hunk.header}:${hunkIndex}`">
          <tr>
            <td class="gl-diff-hunk px-3 py-1 gl-selectable" colspan="4">{{ hunk.header }}</td>
          </tr>
          <tr v-for="(line, lineIndex) in hunk.lines"
              :key="`${hunk.header}:${hunkIndex}:${lineIndex}`"
              class="gl-diff-row"
              :class="rowClass(line)">
            <td class="gl-diff-line-no">{{ line.oldNumber ?? "" }}</td>
            <td class="gl-diff-line-no">{{ line.newNumber ?? "" }}</td>
            <td class="gl-diff-mark">{{ mark(line) }}</td>
            <td class="gl-diff-code">{{ line.content }}</td>
          </tr>
        </tbody>
      </table>
      <table v-else class="gl-diff-split-table">
        <tbody v-for="(hunk, hunkIndex) in splitHunks" :key="`${hunk.header}:${hunkIndex}`">
          <tr>
            <td class="gl-diff-hunk px-3 py-1 gl-selectable" colspan="4">{{ hunk.header }}</td>
          </tr>
          <tr v-for="(row, rowIndex) in hunk.rows"
              :key="`${hunk.header}:${hunkIndex}:${rowIndex}`"
              class="gl-diff-split-row">
            <td class="gl-diff-line-no">{{ row.left.number ?? "" }}</td>
            <td class="gl-diff-split-code gl-selectable" :class="cellClass(row.left)">
              <template v-for="segment in row.left.segments" :key="`${segment.changed}:${segment.text}`">
                <span :class="{ 'gl-diff-inline-change': segment.changed }">{{ segment.text }}</span>
              </template>
            </td>
            <td class="gl-diff-line-no">{{ row.right.number ?? "" }}</td>
            <td class="gl-diff-split-code gl-selectable" :class="cellClass(row.right)">
              <template v-for="segment in row.right.segments" :key="`${segment.changed}:${segment.text}`">
                <span :class="{ 'gl-diff-inline-change': segment.changed }">{{ segment.text }}</span>
              </template>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </section>
</template>
