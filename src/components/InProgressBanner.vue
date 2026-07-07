<script setup lang="ts">
import { computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import {
  AlertTriangle,
  GitMerge,
  GitPullRequestArrow,
  RotateCcw,
} from "lucide-vue-next";
import { useRepoStateStore } from "../stores/repoState";
import { useReposStore } from "../stores/repos";
import type { RepoSnapshot } from "../types/git";

const state = useRepoStateStore();
const repos = useReposStore();
const op = computed(() => state.snapshot?.inProgress ?? null);

async function call(cmd: string) {
  if (!repos.activeId) return;
  try { state.snapshot = await invoke<RepoSnapshot>(cmd, { id: repos.activeId }); }
  catch (e: any) { state.pushToast("error", e?.data?.friendly ?? String(e)); }
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
    <template v-if="op.type === 'merging'">
      <button class="gl-command-btn" @click="call('merge_abort_cmd')">Abort</button>
      <button class="gl-command-btn gl-btn-primary" @click="call('merge_continue_cmd')">Continue</button>
    </template>
    <template v-else-if="op.type === 'rebasing'">
      <button class="gl-command-btn" @click="call('rebase_abort_cmd')">Abort</button>
      <button class="gl-command-btn gl-btn-primary" @click="call('rebase_continue_cmd')">Continue</button>
    </template>
    <template v-else-if="op.type === 'cherryPicking'">
      <button class="gl-command-btn" @click="call('cherry_pick_abort_cmd')">Abort</button>
      <button class="gl-command-btn gl-btn-primary" @click="call('cherry_pick_continue_cmd')">Continue</button>
    </template>
    <template v-else-if="op.type === 'reverting'">
      <button class="gl-command-btn" @click="call('revert_abort_cmd')">Abort</button>
      <button class="gl-command-btn gl-btn-primary" @click="call('revert_continue_cmd')">Continue</button>
    </template>
  </div>
</template>
