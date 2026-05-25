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
  <div v-if="op" class="flex items-center gap-3 px-4 py-2 mx-2 mt-2 rounded-lg"
       style="background: rgba(251, 191, 36, 0.08); border: 1px solid rgba(251, 191, 36, 0.25);">
    <span class="w-2 h-2 rounded-full animate-pulse" style="background: var(--warning)" />
    <span v-if="op.type === 'merging'" class="text-[12.5px]">
      <span style="color: var(--warning)" class="font-semibold">Merging</span>
      <span style="color: var(--fg-2)"> from </span>
      <span class="gl-mono">{{ op.from }}</span>
      <span style="color: var(--fg-3)"> — resolve conflicts then continue</span>
    </span>
    <span v-if="op.type === 'rebasing'" class="text-[12.5px]">
      <span style="color: var(--warning)" class="font-semibold">Rebasing</span>
      <span class="gl-mono"> {{ op.head }}</span>
      <span style="color: var(--fg-3)"> onto </span>
      <span class="gl-mono">{{ op.onto }}</span>
    </span>
    <span v-if="op.type === 'cherryPicking'" class="text-[12.5px]">
      <span style="color: var(--warning)" class="font-semibold">Cherry-picking</span>
      <span style="color: var(--fg-3)"> — resolve conflicts then continue</span>
    </span>
    <span v-if="op.type === 'reverting'" class="text-[12.5px]">
      <span style="color: var(--warning)" class="font-semibold">Reverting</span>
      <span style="color: var(--fg-3)"> — resolve conflicts then continue</span>
    </span>
    <div class="flex-1" />
    <template v-if="op.type === 'merging'">
      <button class="gl-btn gl-btn-ghost" @click="call('merge_abort_cmd')">Abort</button>
      <button class="gl-btn gl-btn-primary" @click="call('merge_continue_cmd')">Continue</button>
    </template>
    <template v-else-if="op.type === 'rebasing'">
      <button class="gl-btn gl-btn-ghost" @click="call('rebase_abort_cmd')">Abort</button>
      <button class="gl-btn gl-btn-primary" @click="call('rebase_continue_cmd')">Continue</button>
    </template>
    <template v-else-if="op.type === 'cherryPicking'">
      <button class="gl-btn gl-btn-ghost" @click="call('cherry_pick_abort_cmd')">Abort</button>
      <button class="gl-btn gl-btn-primary" @click="call('cherry_pick_continue_cmd')">Continue</button>
    </template>
    <template v-else-if="op.type === 'reverting'">
      <button class="gl-btn gl-btn-ghost" @click="call('revert_abort_cmd')">Abort</button>
      <button class="gl-btn gl-btn-primary" @click="call('revert_continue_cmd')">Continue</button>
    </template>
  </div>
</template>
