<script setup lang="ts">
import { computed, reactive, ref } from "vue";
import { useRepoStateStore } from "../stores/repoState";
import { ops } from "../api/tauri";
import { useReposStore } from "../stores/repos";

const state = useRepoStateStore();
const repos = useReposStore();
const selected = reactive<Record<string, boolean>>({});
const message = ref("");
const skipHooks = ref(false);

const files = computed(() => state.snapshot?.files ?? []);
const checkedFiles = computed(() => files.value.filter(f => selected[f.path]).map(f => f.path));

function toggleAll(on: boolean) {
  for (const f of files.value) selected[f.path] = on;
}

async function doCommit() {
  if (!repos.activeId) return;
  try {
    state.snapshot = await ops.commit(repos.activeId, checkedFiles.value, message.value, skipHooks.value);
    message.value = ""; for (const f of files.value) selected[f.path] = false;
  } catch (e: any) { state.pushToast("error", e?.data?.friendly ?? String(e)); }
}

const statusMeta: Record<string, { letter: string; color: string; bg: string }> = {
  modified:   { letter: "M", color: "var(--status-modified)",   bg: "var(--status-modified-bg)" },
  added:      { letter: "A", color: "var(--status-added)",      bg: "var(--status-added-bg)" },
  deleted:    { letter: "D", color: "var(--status-deleted)",    bg: "var(--status-deleted-bg)" },
  renamed:    { letter: "R", color: "var(--status-renamed)",    bg: "var(--status-renamed-bg)" },
  untracked:  { letter: "U", color: "var(--status-untracked)",  bg: "var(--status-untracked-bg)" },
  conflicted: { letter: "!", color: "var(--status-conflicted)", bg: "var(--status-conflicted-bg)" },
};
function statusOf(s: string) { return statusMeta[s] ?? { letter: s[0]?.toUpperCase() ?? "·", color: "var(--fg-2)", bg: "var(--hover)" }; }
</script>

<template>
  <div class="flex flex-col h-full">
    <div class="flex items-center gap-2 px-3 pt-3 pb-2">
      <span class="gl-section-title">Changes</span>
      <span class="gl-chip">{{ files.length }}</span>
      <div class="flex-1" />
      <button class="text-[12px] transition-colors" style="color: var(--fg-3)"
              @click="toggleAll(true)"
              @mouseover="(e: any) => (e.currentTarget.style.color = 'var(--fg)')"
              @mouseleave="(e: any) => (e.currentTarget.style.color = 'var(--fg-3)')">All</button>
      <span style="color: var(--fg-3)">·</span>
      <button class="text-[12px] transition-colors" style="color: var(--fg-3)"
              @click="toggleAll(false)"
              @mouseover="(e: any) => (e.currentTarget.style.color = 'var(--fg)')"
              @mouseleave="(e: any) => (e.currentTarget.style.color = 'var(--fg-3)')">None</button>
    </div>
    <ul class="flex-1 overflow-auto px-2 flex flex-col gap-0.5">
      <li v-for="f in files" :key="f.path"
          class="flex items-center gap-2 px-2 h-7 rounded-md cursor-pointer transition-colors"
          @click="selected[f.path] = !selected[f.path]"
          :style="selected[f.path] ? 'background: var(--accent-soft)' : ''"
          @mouseover="(e: any) => { if (!selected[f.path]) e.currentTarget.style.background = 'var(--hover)' }"
          @mouseleave="(e: any) => { if (!selected[f.path]) e.currentTarget.style.background = '' }">
        <input type="checkbox" :checked="selected[f.path]" @click.stop
               @change="selected[f.path] = !selected[f.path]"
               class="w-3.5 h-3.5 rounded gl-checkbox" />
        <span class="inline-flex items-center justify-center w-5 h-5 rounded text-[11px] font-bold gl-mono"
              :style="{ background: statusOf(f.status).bg, color: statusOf(f.status).color }">
          {{ statusOf(f.status).letter }}
        </span>
        <span class="truncate flex-1 text-[13px] gl-mono" style="color: var(--fg-2)">{{ f.path }}</span>
      </li>
      <li v-if="files.length === 0"
          class="flex flex-col items-center justify-center gap-1 py-8 text-center"
          style="color: var(--fg-3)">
        <span class="text-2xl">✓</span>
        <span class="text-[13px]">Working tree clean</span>
      </li>
    </ul>
    <div class="p-3 flex flex-col gap-2" style="border-top: 1px solid var(--border-soft)">
      <textarea v-model="message" rows="3" placeholder="Commit message…  (⌘K to focus)"
                class="gl-input resize-none gl-mono text-[13px]" />
      <div class="flex items-center justify-between gap-2">
        <label class="flex items-center gap-1.5 text-[12px]" style="color: var(--fg-2)">
          <input type="checkbox" v-model="skipHooks" class="w-3.5 h-3.5 rounded gl-checkbox" />
          Skip hooks (-n)
        </label>
        <button class="gl-btn gl-btn-primary"
                :disabled="checkedFiles.length === 0 || !message.trim()"
                @click="doCommit">
          Commit {{ checkedFiles.length ? `(${checkedFiles.length})` : "" }}
        </button>
      </div>
    </div>
  </div>
</template>
