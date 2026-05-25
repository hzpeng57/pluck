<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from "vue";
import { useReposStore } from "../stores/repos";
import { useRepoStateStore } from "../stores/repoState";
import { invoke } from "@tauri-apps/api/core";
import type { RepoSnapshot } from "../types/git";

const repos = useReposStore();
const state = useRepoStateStore();
const showPushMenu = ref(false);

async function push(force: boolean) {
  if (!repos.activeId) return;
  showPushMenu.value = false;
  try { state.snapshot = await invoke<RepoSnapshot>("push_branch", { id: repos.activeId, forceWithLease: force }); }
  catch (e: any) { state.pushToast("error", e?.data?.friendly ?? String(e)); }
}

async function fetch() {
  if (!repos.activeId) return;
  try { state.snapshot = await invoke<RepoSnapshot>("fetch", { id: repos.activeId }); }
  catch (e: any) { state.pushToast("error", e?.data?.friendly ?? String(e)); }
}

async function pull() {
  const b = state.snapshot?.head.branch; if (!b || !repos.activeId) return;
  try { state.snapshot = await invoke<RepoSnapshot>("pull", { id: repos.activeId, targetBranch: b }); }
  catch (e: any) { state.pushToast("error", e?.data?.friendly ?? String(e)); }
}

const closeMenu = () => { showPushMenu.value = false; };
onMounted(() => window.addEventListener("click", closeMenu));
onBeforeUnmount(() => window.removeEventListener("click", closeMenu));
</script>

<template>
  <header class="flex items-center gap-3 h-12 px-4 shrink-0"
          style="background: var(--bg); border-bottom: 1px solid var(--border-soft)">
    <div class="flex items-center gap-2 min-w-0">
      <div class="w-2 h-2 rounded-full" :style="{ background: state.snapshot ? 'var(--success)' : 'var(--fg-3)' }" />
      <span class="font-semibold truncate" style="color: var(--fg)">{{ repos.active?.name ?? "No repository" }}</span>
      <span style="color: var(--fg-3)">/</span>
      <span class="gl-mono text-[12px] px-1.5 py-0.5 rounded"
            :style="{ background: 'var(--accent-soft)', color: 'var(--accent-2)' }">
        {{ state.snapshot?.head.branch ?? "no head" }}
      </span>
    </div>
    <div class="flex-1" />
    <div class="flex items-center gap-1.5">
      <button class="gl-btn" @click="fetch" title="⌘T">
        <span class="text-[14px] leading-none">↓</span> Fetch
      </button>
      <button class="gl-btn" @click="pull" title="Pull --rebase">
        <span class="text-[14px] leading-none">⇣</span> Pull
      </button>
      <div class="relative">
        <button class="gl-btn gl-btn-primary" @click.stop="showPushMenu = !showPushMenu" title="⌘⇧K">
          <span class="text-[14px] leading-none">↑</span> Push
          <span class="opacity-70 text-[10px] ml-0.5">▾</span>
        </button>
        <div v-if="showPushMenu" class="gl-menu" style="position: absolute; top: 32px; right: 0;">
          <button class="gl-menu-item" @click="push(false)">Push</button>
          <button class="gl-menu-item is-danger" @click="push(true)">Push (force-with-lease)</button>
        </div>
      </div>
    </div>
  </header>
</template>
