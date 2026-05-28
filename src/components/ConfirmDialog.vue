<script setup lang="ts">
import { computed, nextTick, ref, watch } from "vue";
import { useRepoStateStore } from "../stores/repoState";

const state = useRepoStateStore();
const dialog = computed(() => state.confirmDialog);
const text = ref("");
const input = ref<HTMLInputElement | null>(null);
const confirmBtn = ref<HTMLButtonElement | null>(null);

const requiresText = computed(() => !!dialog.value?.confirmText);
const canConfirm = computed(() => !dialog.value?.confirmText || text.value === dialog.value.confirmText);
const toneColor = computed(() => dialog.value?.tone === "danger" ? "var(--danger)" : "var(--warning)");

watch(dialog, async (v) => {
  text.value = "";
  if (!v) return;
  await nextTick();
  if (v.confirmText) input.value?.focus();
  else confirmBtn.value?.focus();
});

function cancel() {
  state.resolveConfirm(false);
}

function confirm() {
  if (!canConfirm.value) return;
  state.resolveConfirm(true);
}

function onKey(e: KeyboardEvent) {
  if (e.key === "Escape") {
    e.preventDefault();
    cancel();
  } else if (e.key === "Enter") {
    e.preventDefault();
    confirm();
  }
}
</script>

<template>
  <div v-if="dialog"
       class="fixed inset-0 flex items-center justify-center z-50"
       style="background: var(--overlay); backdrop-filter: blur(4px)"
       @keydown="onKey">
    <div class="rounded-xl w-[480px] flex flex-col overflow-hidden"
         style="background: var(--panel); border: 1px solid var(--border); box-shadow: var(--shadow-elev)">
      <div class="flex items-center gap-2 px-4 h-12 shrink-0" style="border-bottom: 1px solid var(--border-soft)">
        <span class="w-2 h-2 rounded-full" :style="{ background: toneColor }" />
        <span class="font-semibold text-[13.5px]">{{ dialog.title }}</span>
      </div>

      <div class="px-4 py-3 flex flex-col gap-3">
        <p class="text-[13px] whitespace-pre-wrap" style="color: var(--fg-2)">{{ dialog.message }}</p>
        <label v-if="requiresText" class="flex flex-col gap-1.5">
          <span class="text-[12px]" style="color: var(--fg-3)">
            Type <span class="gl-mono" style="color: var(--fg)">{{ dialog.confirmText }}</span> to confirm
          </span>
          <input ref="input" v-model="text" class="gl-input gl-mono text-[13px]" />
        </label>
      </div>

      <div class="flex items-center gap-2 px-4 py-3 shrink-0" style="border-top: 1px solid var(--border-soft)">
        <div class="flex-1" />
        <button class="gl-btn gl-btn-ghost" @click="cancel">
          {{ dialog.cancelLabel ?? "Cancel" }}
        </button>
        <button ref="confirmBtn"
                class="gl-btn"
                :class="dialog.tone === 'danger' ? 'gl-btn-danger' : 'gl-btn-primary'"
                :disabled="!canConfirm"
                @click="confirm">
          {{ dialog.confirmLabel ?? "Confirm" }}
        </button>
      </div>
    </div>
  </div>
</template>
