<script setup lang="ts">
import { computed, ref, watch, nextTick } from "vue";
import { useRepoStateStore } from "../stores/repoState";
import { useReposStore } from "../stores/repos";
import { useBranchPrefsStore } from "../stores/branchPrefs";
import { ops } from "../api/tauri";

const state = useRepoStateStore();
const repos = useReposStore();
const prefs = useBranchPrefsStore();

const visible = computed(() => state.branchRenameDialog !== null);
const dialog = computed(() => state.branchRenameDialog);
const name = ref("");
const unsetUpstream = ref(false);
const submitting = ref(false);
const input = ref<HTMLInputElement | null>(null);

watch(visible, async (v) => {
  if (!v) {
    name.value = "";
    unsetUpstream.value = false;
    submitting.value = false;
    return;
  }
  name.value = dialog.value?.name ?? "";
  unsetUpstream.value = false;
  submitting.value = false;
  await nextTick();
  input.value?.focus();
  input.value?.select();
});

const trimmedName = computed(() => name.value.trim());
const canRename = computed(() =>
  !!dialog.value && !!trimmedName.value && trimmedName.value !== dialog.value.name && !submitting.value
);

async function submit() {
  if (!repos.activeId || !dialog.value || !canRename.value) return;
  const id = repos.activeId;
  const oldName = dialog.value.name;
  const newName = trimmedName.value;
  submitting.value = true;
  try {
    const wasSelected = state.selectedLogBranch === oldName;
    const next = await ops.branchRename(id, oldName, newName, unsetUpstream.value);
    prefs.renamePinned(id, oldName, newName);
    if (wasSelected) state.selectedLogBranch = newName;
    state.snapshot = next;
    state.closeBranchRenameDialog();
    state.pushToast("info", `Renamed branch "${oldName}" to "${newName}"`);
  } catch (e: any) {
    state.pushToast("error", e?.data?.friendly ?? String(e));
  } finally { submitting.value = false; }
}

function cancel() { if (!submitting.value) state.closeBranchRenameDialog(); }

function onKey(e: KeyboardEvent) {
  if (e.key === "Escape") { e.preventDefault(); cancel(); }
  else if (e.key === "Enter" && canRename.value) { e.preventDefault(); void submit(); }
}
</script>

<template>
  <div v-if="visible && dialog"
       class="fixed inset-0 flex items-center justify-center z-50"
       style="background: var(--overlay); backdrop-filter: blur(4px)"
       @keydown="onKey">
    <div class="rounded-xl w-[520px] flex flex-col overflow-hidden"
         style="background: var(--panel); border: 1px solid var(--border); box-shadow: var(--shadow-elev)">
      <div class="flex items-center gap-2 px-4 h-12 shrink-0" style="border-bottom: 1px solid var(--border-soft)">
        <span class="w-2 h-2 rounded-full" style="background: var(--accent)" />
        <span class="font-semibold text-[13.5px]">Rename Branch</span>
        <span class="gl-mono text-[12px] px-1.5 py-0.5 rounded ml-1 truncate"
              style="background: var(--hover); color: var(--fg-2); max-width: 260px">
          {{ dialog.name }}
        </span>
      </div>

      <div class="p-4 flex flex-col gap-3">
        <label class="flex items-center gap-2 text-[13px]">
          <span class="shrink-0" style="color: var(--fg-2)">Branch Name:</span>
          <input ref="input" v-model="name"
                 type="text"
                 autocapitalize="off"
                 autocorrect="off"
                 autocomplete="off"
                 spellcheck="false"
                 class="gl-input gl-mono text-[13px] flex-1"
                 :disabled="submitting" />
        </label>
        <label v-if="dialog.upstream"
               class="flex items-center gap-2 ml-[92px] text-[13px] cursor-pointer select-none"
               style="color: var(--fg-2)">
          <input type="checkbox" v-model="unsetUpstream" class="cursor-pointer" :disabled="submitting" />
          <span>Unset upstream branch</span>
        </label>
      </div>

      <div class="flex items-center gap-2 px-4 py-3 shrink-0" style="border-top: 1px solid var(--border-soft)">
        <span class="text-[12px]" style="color: var(--fg-3)">
          {{ canRename ? "↩ Rename · Esc Cancel" : "Esc Cancel" }}
        </span>
        <div class="flex-1" />
        <button class="gl-btn gl-btn-ghost" :disabled="submitting" @click="cancel">Cancel</button>
        <button class="gl-btn gl-btn-primary" :disabled="!canRename" @click="submit">
          {{ submitting ? "Renaming…" : "Rename" }}
        </button>
      </div>
    </div>
  </div>
</template>
