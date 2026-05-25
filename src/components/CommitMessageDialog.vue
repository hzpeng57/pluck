<script setup lang="ts">
import { computed, ref, watch, nextTick } from "vue";
import { useRepoStateStore } from "../stores/repoState";
import { useReposStore } from "../stores/repos";
import { api } from "../api/tauri";

const state = useRepoStateStore();
const repos = useReposStore();

const visible = computed(() => state.editMessageDialog !== null);
const dialog = computed(() => state.editMessageDialog);
const message = ref("");
const submitting = ref(false);
const ta = ref<HTMLTextAreaElement | null>(null);

watch(visible, async (v) => {
  if (v && dialog.value) {
    message.value = dialog.value.initial;
    await nextTick();
    ta.value?.focus();
    ta.value?.select();
  }
});

async function save() {
  if (!repos.activeId || !dialog.value || submitting.value) return;
  const text = message.value.trim();
  if (!text) { state.pushToast("error", "Commit message cannot be empty"); return; }
  submitting.value = true;
  try {
    const id = repos.activeId;
    const { hash, mode } = dialog.value;
    if (mode === "amend") state.snapshot = await api.amendHeadMessage(id, text);
    else state.snapshot = await api.rewordAncestor(id, hash, text);
    state.closeEditMessageDialog();
  } catch (e: any) {
    state.pushToast("error", e?.data?.friendly ?? String(e));
  } finally { submitting.value = false; }
}

function cancel() { if (!submitting.value) state.closeEditMessageDialog(); }

function onKey(e: KeyboardEvent) {
  if (e.key === "Escape") { e.preventDefault(); cancel(); }
  else if ((e.metaKey || e.ctrlKey) && e.key === "Enter") { e.preventDefault(); void save(); }
}
</script>

<template>
  <div v-if="visible && dialog"
       class="fixed inset-0 flex items-center justify-center z-50"
       style="background: rgba(11, 13, 16, 0.7); backdrop-filter: blur(4px)"
       @keydown="onKey">
    <div class="rounded-xl w-[640px] flex flex-col overflow-hidden"
         style="background: var(--panel); border: 1px solid var(--border); box-shadow: 0 24px 64px rgba(0,0,0,0.6)">
      <div class="flex items-center gap-2 px-4 h-12 shrink-0" style="border-bottom: 1px solid var(--border-soft)">
        <span class="w-2 h-2 rounded-full" style="background: var(--accent)" />
        <span class="font-semibold text-[13px]">
          {{ dialog.mode === "amend" ? "Amend Commit Message" : "Reword Commit" }}
        </span>
        <span class="gl-mono text-[10.5px] px-1.5 py-0.5 rounded"
              style="background: var(--hover); color: var(--fg-3)">
          {{ dialog.hash.slice(0, 7) }}
        </span>
      </div>

      <div class="flex-1 p-3">
        <textarea ref="ta" v-model="message"
                  class="gl-input gl-mono text-[12.5px] resize-none w-full h-48"
                  placeholder="Commit message…" />
        <p v-if="dialog.mode === 'reword'"
           class="text-[11px] mt-2"
           style="color: var(--warning)">
          ⚠ This rewrites history. The selected commit and every commit after it will get new hashes.
        </p>
      </div>

      <div class="flex items-center gap-2 px-4 py-3 shrink-0" style="border-top: 1px solid var(--border-soft)">
        <span class="text-[11px]" style="color: var(--fg-3)">⌘↩ Save · Esc Cancel</span>
        <div class="flex-1" />
        <button class="gl-btn gl-btn-ghost" :disabled="submitting" @click="cancel">Cancel</button>
        <button class="gl-btn gl-btn-primary" :disabled="submitting" @click="save">
          {{ submitting ? "Saving…" : (dialog.mode === "amend" ? "Amend" : "Reword") }}
        </button>
      </div>
    </div>
  </div>
</template>
