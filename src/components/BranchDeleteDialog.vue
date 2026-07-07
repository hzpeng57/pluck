<script setup lang="ts">
import { computed, ref, watch, nextTick } from "vue";
import { useRepoStateStore } from "../stores/repoState";
import { useReposStore } from "../stores/repos";
import { api, ops } from "../api/tauri";
import type { DeletePrecheck } from "../types/git";

const state = useRepoStateStore();
const repos = useReposStore();

const visible = computed(() => state.branchDeleteDialog !== null);
const dialog = computed(() => state.branchDeleteDialog);
const precheck = ref<DeletePrecheck | null>(null);
const loading = ref(false);
const submitting = ref(false);
const forceChecked = ref(false);
const confirmBtn = ref<HTMLButtonElement | null>(null);

watch(visible, async (v) => {
  if (!v) {
    precheck.value = null;
    forceChecked.value = false;
    submitting.value = false;
    return;
  }
  if (!repos.activeId || !dialog.value) return;
  loading.value = true;
  try {
    const result = await ops.branchDeletePrecheck(repos.activeId, dialog.value.name);
    if (!result.exists) {
      // 分支不存在：刷新快照后静默关闭，给个友好 toast
      try { state.snapshot = await api.repoRefresh(repos.activeId); }
      catch { /* ignore */ }
      state.pushToast("info", `Branch "${dialog.value.name}" no longer exists`);
      state.closeBranchDeleteDialog();
      return;
    }
    precheck.value = result;
    forceChecked.value = false;
    await nextTick();
    if (!result.isCurrent && result.isMerged) confirmBtn.value?.focus();
  } catch (e: any) {
    state.pushToast("error", e?.data?.friendly ?? String(e));
    state.closeBranchDeleteDialog();
  } finally { loading.value = false; }
});

const canDelete = computed(() => {
  if (!precheck.value || precheck.value.isCurrent) return false;
  if (precheck.value.isMerged) return true;
  return forceChecked.value;
});

async function submit() {
  if (!repos.activeId || !dialog.value || !precheck.value || submitting.value) return;
  if (!canDelete.value) return;
  submitting.value = true;
  const force = !precheck.value.isMerged;
  try {
    state.snapshot = await ops.branchDelete(repos.activeId, dialog.value.name, force);
    state.pushToast("info", `Deleted branch "${dialog.value.name}"`);
    state.closeBranchDeleteDialog();
  } catch (e: any) {
    state.pushToast("error", e?.data?.friendly ?? String(e));
  } finally { submitting.value = false; }
}

function cancel() { if (!submitting.value) state.closeBranchDeleteDialog(); }

function onKey(e: KeyboardEvent) {
  if (e.key === "Escape") { e.preventDefault(); cancel(); }
  else if (e.key === "Enter" && canDelete.value) { e.preventDefault(); void submit(); }
}
</script>

<template>
  <div v-if="visible && dialog"
       class="fixed inset-0 flex items-center justify-center z-50 gl-overlay"
       @keydown="onKey">
    <div class="gl-dialog-shell w-[480px] flex flex-col">
      <div class="flex items-center gap-2 px-4 h-12 shrink-0" style="border-bottom: 1px solid var(--border-soft)">
        <span class="w-2 h-2 rounded-full" style="background: var(--danger)" />
        <span class="font-semibold text-[13.5px]">Delete Branch</span>
        <span class="gl-mono text-[12px] px-1.5 py-0.5 rounded ml-1"
              style="background: var(--hover); color: var(--fg-2)">
          {{ dialog.name }}
        </span>
      </div>

      <div class="p-4 text-[13px]" style="color: var(--fg-2)">
        <div v-if="loading" style="color: var(--fg-3)">Checking branch status…</div>

        <template v-else-if="precheck">
          <!-- 当前分支 -->
          <div v-if="precheck.isCurrent"
               class="rounded px-3 py-2"
               style="background: var(--danger-soft-weak); color: var(--danger); border: 1px solid var(--danger-ring)">
            Cannot delete the branch currently checked out. Switch to another branch first.
          </div>

          <!-- 已完全合并 -->
          <div v-else-if="precheck.isMerged">
            <p>This branch is fully merged into HEAD. Safe to delete.</p>
            <p v-if="precheck.upstream" class="mt-2 text-[12px]" style="color: var(--fg-3)">
              Tracks <span class="gl-mono">{{ precheck.upstream }}</span> — remote-tracking ref will be kept.
            </p>
          </div>

          <!-- 未合并 -->
          <div v-else>
            <p>
              <strong>{{ precheck.aheadOfHead }}</strong>
              commit{{ precheck.aheadOfHead === 1 ? "" : "s" }}
              on this branch {{ precheck.aheadOfHead === 1 ? "is" : "are" }} not reachable from HEAD.
              Deleting will lose {{ precheck.aheadOfHead === 1 ? "it" : "them" }}
              (recoverable from reflog for ~90 days).
            </p>
            <label class="flex items-center gap-2 mt-3 cursor-pointer select-none">
              <input type="checkbox" v-model="forceChecked" class="cursor-pointer" />
              <span>I understand — force delete</span>
            </label>
          </div>
        </template>
      </div>

      <div class="flex items-center gap-2 px-4 py-3 shrink-0" style="border-top: 1px solid var(--border-soft)">
        <span class="text-[12px]" style="color: var(--fg-3)">
          {{ canDelete ? "↩ Delete · Esc Cancel" : "Esc Cancel" }}
        </span>
        <div class="flex-1" />
        <button class="gl-command-btn" :disabled="submitting" @click="cancel">Cancel</button>
        <button ref="confirmBtn"
                class="gl-command-btn"
                :class="precheck && !precheck.isMerged ? 'gl-btn-danger' : 'gl-btn-primary'"
                :disabled="!canDelete || submitting"
                @click="submit">
          {{ submitting ? "Deleting…" : (precheck && !precheck.isMerged ? "Force Delete" : "Delete") }}
        </button>
      </div>
    </div>
  </div>
</template>
