<script setup lang="ts">
import { computed } from "vue";
import {
  AlertTriangle,
  GitMerge,
  GitPullRequestArrow,
  RotateCcw,
} from "lucide-vue-next";
import { useRepoStateStore } from "../stores/repoState";
import { useReposStore } from "../stores/repos";

const state = useRepoStateStore();
const repos = useReposStore();
const op = computed(() => state.snapshot?.inProgress ?? null);
const unresolvedCount = computed(() =>
  state.snapshot?.files.filter(file => file.status === "conflicted").length ?? 0,
);

function resolveConflicts() {
  if (repos.activeId) void state.openConflictWorkspace(repos.activeId);
}

function continueOperation() {
  if (repos.activeId) void state.continueInProgress(repos.activeId);
}

function abortOperation() {
  if (repos.activeId) void state.abortInProgress(repos.activeId);
}
</script>

<template>
  <div v-if="op" class="gl-banner mx-3 mt-3 rounded-lg px-3 py-2"
       style="border-color: var(--warning-ring); background: var(--warning-soft-weak)">
    <AlertTriangle :size="16" class="shrink-0" style="color: var(--warning)" />
    <span v-if="op.type === 'merging'" class="text-[13px]">
      <GitMerge :size="14" class="inline shrink-0 mr-1" style="color: var(--warning)" />
      <span style="color: var(--warning)" class="font-semibold">Merging</span>
      <span style="color: var(--fg-2)"> from </span>
      <span class="gl-mono">{{ op.from }}</span>
      <span style="color: var(--fg-3)"> — resolve conflicts then continue</span>
    </span>
    <span v-if="op.type === 'rebasing'" class="text-[13px]">
      <GitPullRequestArrow :size="14" class="inline shrink-0 mr-1" style="color: var(--warning)" />
      <span style="color: var(--warning)" class="font-semibold">Rebasing</span>
      <span class="gl-mono"> {{ op.head }}</span>
      <span style="color: var(--fg-3)"> onto </span>
      <span class="gl-mono">{{ op.onto }}</span>
    </span>
    <span v-if="op.type === 'cherryPicking'" class="text-[13px]">
      <RotateCcw :size="14" class="inline shrink-0 mr-1" style="color: var(--warning)" />
      <span style="color: var(--warning)" class="font-semibold">Cherry-picking</span>
      <span style="color: var(--fg-3)"> — resolve conflicts then continue</span>
    </span>
    <span v-if="op.type === 'reverting'" class="text-[13px]">
      <RotateCcw :size="14" class="inline shrink-0 mr-1" style="color: var(--warning)" />
      <span style="color: var(--warning)" class="font-semibold">Reverting</span>
      <span style="color: var(--fg-3)"> — resolve conflicts then continue</span>
    </span>
    <div class="flex-1" />
    <button v-if="unresolvedCount > 0" class="gl-command-btn" @click="resolveConflicts">
      Resolve {{ unresolvedCount }} {{ unresolvedCount === 1 ? "conflict" : "conflicts" }}
    </button>
    <button class="gl-command-btn" @click="abortOperation">Abort</button>
    <button class="gl-command-btn gl-btn-primary" :disabled="unresolvedCount > 0" @click="continueOperation">Continue</button>
  </div>
</template>
