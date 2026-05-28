<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useRepoStateStore } from "../stores/repoState";
import { useReposStore } from "../stores/repos";
import { api } from "../api/tauri";
import type { ResetMode } from "../types/git";

const state = useRepoStateStore();
const repos = useReposStore();

const visible = computed(() => state.resetDialog !== null);
const dialog = computed(() => state.resetDialog);
const mode = ref<ResetMode>("mixed");
const submitting = ref(false);

watch(visible, (v) => { if (v) mode.value = "mixed"; });

const MODES: { value: ResetMode; label: string; desc: string; danger?: boolean }[] = [
  { value: "soft",  label: "Soft",  desc: "Move HEAD only. Index and working tree are untouched." },
  { value: "mixed", label: "Mixed (default)", desc: "Move HEAD and reset the index. Working tree changes are kept and unstaged." },
  { value: "hard",  label: "Hard",  desc: "Move HEAD, reset the index, and discard ALL working tree changes.", danger: true },
  { value: "keep",  label: "Keep",  desc: "Like hard, but abort if any local change would be lost." },
];

async function submit() {
  if (!repos.activeId || !dialog.value || submitting.value) return;
  if (mode.value === "hard") {
    const ok = await state.confirmAction({
      title: "Confirm Hard Reset",
      message: `Reset --hard to ${dialog.value.short} will discard all uncommitted changes in your working tree.`,
      confirmLabel: "Reset --hard",
      tone: "danger",
    });
    if (!ok) return;
  }
  submitting.value = true;
  try {
    state.snapshot = await api.resetTo(repos.activeId, dialog.value.hash, mode.value);
    state.closeResetDialog();
    state.clearSelection();
  } catch (e: any) {
    state.pushToast("error", e?.data?.friendly ?? String(e));
  } finally { submitting.value = false; }
}

function cancel() { if (!submitting.value) state.closeResetDialog(); }

function onKey(e: KeyboardEvent) {
  if (e.key === "Escape") { e.preventDefault(); cancel(); }
  else if ((e.metaKey || e.ctrlKey) && e.key === "Enter") { e.preventDefault(); void submit(); }
}
</script>

<template>
  <div v-if="visible && dialog"
       class="fixed inset-0 flex items-center justify-center z-50"
       style="background: var(--overlay); backdrop-filter: blur(4px)"
       @keydown="onKey">
    <div class="rounded-xl w-[560px] flex flex-col overflow-hidden"
         style="background: var(--panel); border: 1px solid var(--border); box-shadow: var(--shadow-elev)">
      <div class="flex items-center gap-2 px-4 h-12 shrink-0" style="border-bottom: 1px solid var(--border-soft)">
        <span class="w-2 h-2 rounded-full" style="background: var(--warning)" />
        <span class="font-semibold text-[13.5px]">Reset Current Branch to Here</span>
      </div>

      <div class="px-4 py-3">
        <div class="flex items-center gap-2 mb-3 text-[13px]" style="color: var(--fg-2)">
          <span>Target:</span>
          <span class="gl-mono text-[12px] px-1.5 py-0.5 rounded"
                style="background: var(--hover); color: var(--fg-3)">{{ dialog.short }}</span>
          <span class="truncate flex-1" style="color: var(--fg)">{{ dialog.subject }}</span>
        </div>

        <div class="flex flex-col gap-2">
          <label v-for="m in MODES" :key="m.value"
                 class="flex items-start gap-2.5 px-3 py-2 rounded-md cursor-pointer transition-colors"
                 :style="mode === m.value
                    ? { background: 'var(--accent-soft)', border: '1px solid var(--accent)' }
                    : { background: 'var(--bg)', border: '1px solid var(--border)' }">
            <input type="radio" :value="m.value" v-model="mode" class="mt-1" />
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-2">
                <span class="font-medium text-[13px]"
                      :style="{ color: m.danger ? 'var(--danger)' : 'var(--fg)' }">{{ m.label }}</span>
                <span v-if="m.danger" class="gl-mono text-[11px] px-1.5 py-0.5 rounded"
                      style="background: var(--danger); color: var(--on-danger)">DESTRUCTIVE</span>
              </div>
              <div class="text-[12px] mt-0.5" style="color: var(--fg-3)">{{ m.desc }}</div>
            </div>
          </label>
        </div>
      </div>

      <div class="flex items-center gap-2 px-4 py-3 shrink-0" style="border-top: 1px solid var(--border-soft)">
        <span class="text-[12px]" style="color: var(--fg-3)">⌘↩ Reset · Esc Cancel</span>
        <div class="flex-1" />
        <button class="gl-btn gl-btn-ghost" :disabled="submitting" @click="cancel">Cancel</button>
        <button class="gl-btn"
                :class="mode === 'hard' ? 'gl-btn-danger' : 'gl-btn-primary'"
                :disabled="submitting" @click="submit">
          {{ submitting ? "Resetting…" : "Reset" }}
        </button>
      </div>
    </div>
  </div>
</template>
