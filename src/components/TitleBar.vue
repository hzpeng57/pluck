<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from "vue";
import { useReposStore } from "../stores/repos";
import { useRepoStateStore } from "../stores/repoState";
import { invoke } from "@tauri-apps/api/core";
import type { RepoSnapshot } from "../types/git";

const repos = useReposStore();
const state = useRepoStateStore();
const showPushMenu = ref(false);

const fetching = ref(false);
const pulling = ref(false);
const pushing = ref(false);
const copied = ref(false);

async function copyBranch() {
  const b = state.snapshot?.head.branch;
  if (!b) return;
  try {
    await navigator.clipboard.writeText(b);
    copied.value = true;
    setTimeout(() => { copied.value = false; }, 1200);
  } catch (e: any) { state.pushToast("error", `Copy failed: ${e?.message ?? e}`); }
}

async function push(force: boolean) {
  if (!repos.activeId || pushing.value) return;
  showPushMenu.value = false;
  pushing.value = true;
  try {
    state.snapshot = await invoke<RepoSnapshot>("push_branch", { id: repos.activeId, forceWithLease: force });
    state.pushToast("info", force ? "Push (force-with-lease) successful" : "Push successful");
  } catch (e: any) { state.pushToast("error", e?.data?.friendly ?? String(e)); }
  finally { pushing.value = false; }
}

async function fetch() {
  if (!repos.activeId || fetching.value) return;
  fetching.value = true;
  try {
    state.snapshot = await invoke<RepoSnapshot>("fetch", { id: repos.activeId });
    state.pushToast("info", "Fetch successful");
  } catch (e: any) { state.pushToast("error", e?.data?.friendly ?? String(e)); }
  finally { fetching.value = false; }
}

async function pull() {
  const b = state.snapshot?.head.branch; if (!b || !repos.activeId || pulling.value) return;
  pulling.value = true;
  try {
    state.snapshot = await invoke<RepoSnapshot>("pull", { id: repos.activeId, targetBranch: b });
    state.pushToast("info", "Pull successful");
  } catch (e: any) { state.pushToast("error", e?.data?.friendly ?? String(e)); }
  finally { pulling.value = false; }
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
      <span class="gl-mono text-[13px] px-1.5 py-0.5 rounded"
            :style="{ background: 'var(--accent-soft)', color: 'var(--accent-2)' }">
        {{ state.snapshot?.head.branch ?? "no head" }}
      </span>
      <button v-if="state.snapshot?.head.branch"
              class="gl-icon-btn"
              :title="copied ? 'Copied!' : 'Copy branch name'"
              @click="copyBranch">
        <svg v-if="!copied" width="13" height="13" viewBox="0 0 24 24" fill="none"
             stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
          <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
        </svg>
        <svg v-else width="13" height="13" viewBox="0 0 24 24" fill="none"
             :stroke="'var(--success)'" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="20 6 9 17 4 12" />
        </svg>
      </button>
    </div>
    <div class="flex-1" />
    <div class="flex items-center gap-1.5">
      <button class="gl-btn" @click="fetch" :disabled="fetching" title="⌘T">
        <span v-if="fetching" class="gl-spinner" />
        <span v-else class="text-[14.5px] leading-none">↓</span>
        {{ fetching ? "Fetching…" : "Fetch" }}
      </button>
      <button class="gl-btn" @click="pull" :disabled="pulling" title="Pull --rebase">
        <span v-if="pulling" class="gl-spinner" />
        <span v-else class="text-[14.5px] leading-none">⇣</span>
        {{ pulling ? "Pulling…" : "Pull" }}
      </button>
      <div class="relative">
        <button class="gl-btn gl-btn-primary" @click.stop="showPushMenu = !showPushMenu"
                :disabled="pushing" title="⌘⇧K">
          <span v-if="pushing" class="gl-spinner" />
          <span v-else class="text-[14.5px] leading-none">↑</span>
          {{ pushing ? "Pushing…" : "Push" }}
          <span v-if="!pushing" class="opacity-70 text-[11px] ml-0.5">▾</span>
        </button>
        <div v-if="showPushMenu" class="gl-menu" style="position: absolute; top: 32px; right: 0;">
          <button class="gl-menu-item" @click="push(false)">Push</button>
          <button class="gl-menu-item is-danger" @click="push(true)">Push (force-with-lease)</button>
        </div>
      </div>
    </div>
  </header>
</template>
