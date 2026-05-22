<script setup lang="ts">
import { computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
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
  <div v-if="op" class="px-3 py-2 bg-amber-200 dark:bg-amber-900 text-sm flex items-center gap-3">
    <span v-if="op.type === 'merging'">Merge in progress from {{ op.from }}. Resolve conflicts in your editor.</span>
    <span v-if="op.type === 'rebasing'">Rebase in progress ({{ op.head }} onto {{ op.onto }}). Resolve conflicts in your editor.</span>
    <span v-if="op.type === 'cherryPicking'">Cherry-pick in progress.</span>
    <div class="flex-1" />
    <template v-if="op.type === 'merging'">
      <button class="px-2 py-0.5 rounded border" @click="call('merge_continue_cmd')">Continue</button>
      <button class="px-2 py-0.5 rounded border" @click="call('merge_abort_cmd')">Abort</button>
    </template>
    <template v-else-if="op.type === 'rebasing'">
      <button class="px-2 py-0.5 rounded border" @click="call('rebase_continue_cmd')">Continue</button>
      <button class="px-2 py-0.5 rounded border" @click="call('rebase_abort_cmd')">Abort</button>
    </template>
  </div>
</template>
