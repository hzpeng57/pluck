<script setup lang="ts">
import { computed, ref } from "vue";
import { ArrowLeft, RotateCcw, WrapText } from "lucide-vue-next";
import { useRepoStateStore } from "../stores/repoState";
import type { DiffLine, DiffTarget, FileDiff } from "../types/git";

const emit = defineEmits<{
  back: [];
  rollback: [];
}>();

const state = useRepoStateStore();
const wrap = ref(false);
const diff = computed(() => state.selectedDiff);
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
        <span class="min-w-0 flex-1 truncate gl-mono text-[12px] gl-selectable" :title="displayPath">
          {{ displayPath }}
        </span>
        <template v-if="diff">
          <span class="gl-badge" style="color: var(--success)">+{{ diff.additions }}</span>
          <span class="gl-badge" style="color: var(--danger)">-{{ diff.deletions }}</span>
          <span v-if="diff.tooLarge" class="gl-badge" style="color: var(--warning)">truncated</span>
        </template>
      </template>
      <div v-else class="flex-1" />

      <button class="gl-command-btn h-7 px-2"
              :class="{ 'gl-btn-primary': wrap }"
              :aria-pressed="wrap"
              title="Toggle line wrap"
              @click="wrap = !wrap">
        <WrapText :size="13" />
        Wrap
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
      <span class="gl-mono text-[12px] gl-selectable">{{ diff.path }}</span>
    </div>
    <div v-else-if="diff.hunks.length === 0" class="gl-empty flex-1">
      <span class="text-[13px]">No textual changes</span>
      <span class="gl-mono text-[12px] gl-selectable">{{ diff.path }}</span>
    </div>
    <div v-else class="gl-diff-scroll" :class="{ 'is-wrap': wrap }">
      <table class="gl-diff-table">
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
    </div>
  </section>
</template>
