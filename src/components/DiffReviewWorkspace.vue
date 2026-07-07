<script setup lang="ts">
import { computed } from "vue";
import { useReposStore } from "../stores/repos";
import { useRepoStateStore } from "../stores/repoState";
import CommitPanel from "./CommitPanel.vue";
import CommitDetailPanel from "./CommitDetailPanel.vue";
import DiffViewer from "./DiffViewer.vue";

const repos = useReposStore();
const state = useRepoStateStore();
const sourceIsCommit = computed(() => state.diffTarget?.kind === "commit");

async function rollback() {
  if (!repos.activeId || state.diffTarget?.kind !== "workingTree") return;
  const repoId = repos.activeId;
  const target = state.diffTarget;
  const destructive = target.status === "untracked" || target.status === "added" || target.status === "renamed";
  const ok = await state.confirmAction({
    title: "Rollback File",
    message: destructive
      ? `Rollback will remove "${target.path}" from your working tree.`
      : `Rollback will restore "${target.path}" to HEAD.`,
    confirmLabel: "Rollback",
    tone: destructive ? "danger" : "warning",
    confirmText: destructive ? target.path : undefined,
  });
  if (!ok) return;
  const current = state.diffTarget;
  if (
    !repos.activeId ||
    repos.activeId !== repoId ||
    current?.kind !== "workingTree" ||
    current.path !== target.path ||
    current.oldPath !== target.oldPath ||
    current.status !== target.status
  ) return;
  try {
    await state.rollbackCurrentWorkingFile(repoId);
    if (state.diffError) {
      state.pushToast("error", state.diffError);
      return;
    }
    state.pushToast("info", `Rolled back ${target.path}`);
  } catch (e: any) {
    state.pushToast("error", e?.data?.friendly ?? e?.message ?? String(e));
  }
}
</script>

<template>
  <div class="h-full min-h-0 min-w-0 grid" style="grid-template-columns: minmax(300px, 360px) minmax(620px, 1fr)">
    <aside class="min-h-0 overflow-hidden" style="border-right: 1px solid var(--border-soft); background: var(--panel)">
      <CommitDetailPanel v-if="sourceIsCommit" review-mode />
      <CommitPanel v-else review-mode />
    </aside>
    <DiffViewer @back="state.closeReviewMode()" @rollback="rollback" />
  </div>
</template>
