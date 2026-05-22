<script setup lang="ts">
import { watch } from "vue";
import { useReposStore } from "./stores/repos";
import { useRepoStateStore } from "./stores/repoState";
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
watch(() => repos.activeId, id => { if (id) state.open(id); }, { immediate: true });
window.addEventListener("focus", () => { if (repos.activeId) state.refresh(repos.activeId); });
</script>

<template>
  <div class="flex h-full bg-neutral-50 dark:bg-neutral-950 text-sm">
    <RepoSwitcher class="w-44 shrink-0 border-r border-neutral-200 dark:border-neutral-800" />
    <div class="flex-1 flex flex-col min-w-0">
      <TitleBar />
      <InProgressBanner v-if="state.snapshot?.inProgress" />
      <div class="flex-1 grid grid-cols-[260px_1fr] grid-rows-[1fr_1fr] gap-px bg-neutral-200 dark:bg-neutral-800 overflow-hidden">
        <BranchesPanel class="row-span-2 bg-neutral-50 dark:bg-neutral-900 overflow-auto" />
        <CommitPanel class="bg-neutral-50 dark:bg-neutral-900 overflow-auto" />
        <LogPanel class="bg-neutral-50 dark:bg-neutral-900 overflow-auto" />
      </div>
      <StatusBar />
    </div>
    <ToastTray />
    <RebaseTodoDialog />
  </div>
</template>
