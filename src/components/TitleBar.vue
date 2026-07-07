<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from "vue";
import {
  Check,
  ChevronDown,
  Clipboard,
  Download,
  GitBranch,
  GitPullRequestArrow,
  Upload,
} from "lucide-vue-next";
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
  const branch = state.snapshot?.head.branch;
  showPushMenu.value = false;
  if (force) {
    if (!branch) return;
    const ok = await state.confirmAction({
      title: "Confirm Force Push",
      message: `Force-with-lease can rewrite the remote branch origin/${branch}.`,
      confirmLabel: "Force push",
      tone: "danger",
      confirmText: branch,
    });
    if (!ok) return;
  }
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
    const snapshot = await invoke<RepoSnapshot>("pull", { id: repos.activeId, targetBranch: b });
    state.snapshot = snapshot;
    state.pushToast("info", snapshot.inProgress?.type === "rebasing"
      ? "Pull paused for conflicts"
      : "Pull successful");
  } catch (e: any) { state.pushToast("error", e?.data?.friendly ?? String(e)); }
  finally { pulling.value = false; }
}

const closeMenu = () => { showPushMenu.value = false; };
onMounted(() => window.addEventListener("click", closeMenu));
onBeforeUnmount(() => window.removeEventListener("click", closeMenu));
</script>

<template>
  <header class="gl-toolbar shrink-0">
    <div class="flex items-center gap-2 min-w-0">
      <span class="gl-status-dot"
            :style="{ color: state.snapshot ? 'var(--success)' : 'var(--fg-3)', background: state.snapshot ? 'var(--success)' : 'var(--fg-3)' }" />
      <div class="min-w-0">
        <div class="flex items-center gap-1.5 min-w-0">
          <span class="font-semibold truncate text-[13.5px]" style="color: var(--fg)">
            {{ repos.active?.name ?? "No repository" }}
          </span>
          <span style="color: var(--fg-3)">/</span>
          <span class="gl-badge">
            <GitBranch :size="12" />
            {{ state.snapshot?.head.branch ?? "no head" }}
          </span>
        </div>
      </div>
      <button v-if="state.snapshot?.head.branch"
              class="gl-icon-btn"
              :title="copied ? 'Copied!' : 'Copy branch name'"
              @click="copyBranch">
        <Clipboard v-if="!copied" :size="14" />
        <Check v-else :size="14" style="color: var(--success)" />
      </button>
    </div>
    <div class="flex-1" />
    <div class="flex items-center gap-1.5">
      <button class="gl-command-btn" @click="fetch" :disabled="fetching" title="⌘T">
        <span v-if="fetching" class="gl-spinner" />
        <Download v-else :size="14" />
        {{ fetching ? "Fetching" : "Fetch" }}
      </button>
      <button class="gl-command-btn" @click="pull" :disabled="pulling" title="Pull --rebase">
        <span v-if="pulling" class="gl-spinner" />
        <GitPullRequestArrow v-else :size="14" />
        {{ pulling ? "Pulling" : "Pull" }}
      </button>
      <div class="relative">
        <button class="gl-command-btn gl-btn-primary" @click.stop="showPushMenu = !showPushMenu"
                :disabled="pushing" title="⌘⇧K">
          <span v-if="pushing" class="gl-spinner" />
          <Upload v-else :size="14" />
          {{ pushing ? "Pushing" : "Push" }}
          <ChevronDown v-if="!pushing" :size="13" class="opacity-70" />
        </button>
        <div v-if="showPushMenu" class="gl-menu" style="position: absolute; top: 34px; right: 0;">
          <button class="gl-menu-item" @click="push(false)">Push</button>
          <button class="gl-menu-item is-danger" @click="push(true)">Push (force-with-lease)</button>
        </div>
      </div>
    </div>
  </header>
</template>
