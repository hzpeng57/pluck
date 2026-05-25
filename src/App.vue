<script setup lang="ts">
import { watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useReposStore } from "./stores/repos";
import { useRepoStateStore } from "./stores/repoState";
import { api } from "./api/tauri";
import type { RepoSnapshot } from "./types/git";
import RepoSwitcher from "./components/RepoSwitcher.vue";
import TitleBar from "./components/TitleBar.vue";
import StatusBar from "./components/StatusBar.vue";
import BranchesPanel from "./components/BranchesPanel.vue";
import CommitPanel from "./components/CommitPanel.vue";
import LogPanel from "./components/LogPanel.vue";
import InProgressBanner from "./components/InProgressBanner.vue";
import ToastTray from "./components/ToastTray.vue";
import RebaseTodoDialog from "./components/RebaseTodoDialog.vue";

const repos = useReposStore();
const state = useRepoStateStore();
watch(() => repos.activeId, async id => {
  if (!id) return;
  const meta = repos.all.find(r => r.id === id);
  if (meta) {
    try { await api.repoAdd(meta.path); }
    catch (e: any) { state.pushToast("error", e?.data?.friendly ?? String(e)); return; }
  }
  state.open(id);
}, { immediate: true });
window.addEventListener("focus", () => { if (repos.activeId) state.refresh(repos.activeId); });

function isMeta(e: KeyboardEvent) { return e.metaKey || e.ctrlKey; }

window.addEventListener("keydown", async (e) => {
  if (!repos.activeId) return;
  if (isMeta(e) && e.key.toLowerCase() === "k" && !e.shiftKey) {
    e.preventDefault();
    (document.querySelector("textarea") as HTMLTextAreaElement | null)?.focus();
  } else if (isMeta(e) && e.shiftKey && e.key.toLowerCase() === "k") {
    e.preventDefault();
    try { state.snapshot = await invoke<RepoSnapshot>("push_branch", { id: repos.activeId, forceWithLease: false }); }
    catch (err: any) { state.pushToast("error", err?.data?.friendly ?? String(err)); }
  } else if (isMeta(e) && e.key.toLowerCase() === "t") {
    e.preventDefault();
    try { state.snapshot = await invoke<RepoSnapshot>("fetch", { id: repos.activeId }); }
    catch (err: any) { state.pushToast("error", err?.data?.friendly ?? String(err)); }
  } else if (isMeta(e) && e.key.toLowerCase() === "r") {
    e.preventDefault();
    state.refresh(repos.activeId);
  }
});
</script>

<template>
  <div class="flex h-full" style="background: var(--bg); color: var(--fg)">
    <RepoSwitcher />
    <div class="flex-1 flex flex-col min-w-0">
      <TitleBar />
      <InProgressBanner v-if="state.snapshot?.inProgress" />
      <div class="flex-1 grid grid-cols-[260px_1fr] grid-rows-[1fr_1fr] gap-2 p-2 overflow-hidden">
        <div class="row-span-2 gl-surface rounded-lg overflow-auto" style="border: 1px solid var(--border)">
          <BranchesPanel />
        </div>
        <div class="gl-surface rounded-lg overflow-auto" style="border: 1px solid var(--border)">
          <CommitPanel />
        </div>
        <div class="gl-surface rounded-lg overflow-auto" style="border: 1px solid var(--border)">
          <LogPanel />
        </div>
      </div>
      <StatusBar />
    </div>
    <ToastTray />
    <RebaseTodoDialog />
  </div>
</template>
