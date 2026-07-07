<script setup lang="ts">
import { computed, ref, watch, nextTick } from "vue";
import { useRepoStateStore } from "../stores/repoState";
import { useReposStore } from "../stores/repos";
import { ops } from "../api/tauri";

const state = useRepoStateStore();
const repos = useReposStore();

const visible = computed(() => state.branchCreateDialog !== null);
const dialog = computed(() => state.branchCreateDialog);
const name = ref("");
const submitting = ref(false);
const input = ref<HTMLInputElement | null>(null);

watch(visible, async (v) => {
  if (v) {
    name.value = "";
    submitting.value = false;
    await nextTick();
    input.value?.focus();
  }
});

async function submit() {
  if (!repos.activeId || !dialog.value || submitting.value) return;
  const trimmed = name.value.trim();
  if (!trimmed) { state.pushToast("error", "Branch name cannot be empty"); return; }
  const id = repos.activeId;
  const from = dialog.value.from;
  submitting.value = true;
  try {
    state.snapshot = await ops.branchCreate(id, trimmed, from);
    state.closeBranchCreateDialog();
    state.pushToast("info", `Created branch "${trimmed}"`);
  } catch (e: any) {
    state.closeBranchCreateDialog();
    state.pushToast("error", e?.data?.friendly ?? String(e));
  } finally { submitting.value = false; }
}

function cancel() { if (!submitting.value) state.closeBranchCreateDialog(); }

function onKey(e: KeyboardEvent) {
  if (e.key === "Escape") { e.preventDefault(); cancel(); }
  else if (e.key === "Enter") { e.preventDefault(); void submit(); }
}
</script>

<template>
  <div v-if="visible && dialog"
       class="fixed inset-0 flex items-center justify-center z-50 gl-overlay"
       @keydown="onKey">
    <div class="gl-dialog-shell w-[480px] flex flex-col">
      <div class="flex items-center gap-2 px-4 h-12 shrink-0" style="border-bottom: 1px solid var(--border-soft)">
        <span class="w-2 h-2 rounded-full" style="background: var(--accent)" />
        <span class="font-semibold text-[13.5px]">New Branch</span>
        <span class="text-[12px]" style="color: var(--fg-3)">from</span>
        <span class="gl-mono text-[12px] px-1.5 py-0.5 rounded"
              style="background: var(--hover); color: var(--fg-2)">
          {{ dialog.from }}
        </span>
      </div>

      <div class="p-3">
        <input ref="input" v-model="name"
               type="text"
               autocapitalize="off"
               autocorrect="off"
               autocomplete="off"
               spellcheck="false"
               class="gl-input gl-mono text-[13px] w-full"
               placeholder="Branch name…" />
      </div>

      <div class="flex items-center gap-2 px-4 py-3 shrink-0" style="border-top: 1px solid var(--border-soft)">
        <span class="text-[12px]" style="color: var(--fg-3)">↩ Create · Esc Cancel</span>
        <div class="flex-1" />
        <button class="gl-command-btn" :disabled="submitting" @click="cancel">Cancel</button>
        <button class="gl-command-btn gl-btn-primary" :disabled="submitting || !name.trim()" @click="submit">
          {{ submitting ? "Creating…" : "Create" }}
        </button>
      </div>
    </div>
  </div>
</template>
