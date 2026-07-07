<script setup lang="ts">
import { computed, reactive, ref } from "vue";
import {
  CheckCircle2,
  CircleOff,
  GitCommitHorizontal,
} from "lucide-vue-next";
import { useRepoStateStore } from "../stores/repoState";
import { ops } from "../api/tauri";
import { useReposStore } from "../stores/repos";
import type { WorkingFile } from "../types/git";

const props = withDefaults(defineProps<{ reviewMode?: boolean }>(), { reviewMode: false });

const state = useRepoStateStore();
const repos = useReposStore();
const selected = reactive<Record<string, boolean>>({});
const message = ref("");
const skipHooks = ref(false);

const files = computed(() => state.snapshot?.files ?? []);
const checkedFiles = computed(() => files.value.filter(f => selected[f.path]).map(f => f.path));

async function openDiff(f: WorkingFile) {
  if (!repos.activeId) return;
  await state.openWorkingDiff(repos.activeId, f);
}

function isDiffSelected(f: WorkingFile) {
  const target = state.diffTarget;
  return target?.kind === "workingTree" && target.path === f.path;
}

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
    <div class="gl-panel-header">
      <span class="gl-section-title">Changes</span>
      <span class="gl-badge">{{ files.length }}</span>
      <div class="flex-1" />
      <button v-if="!props.reviewMode" class="gl-command-btn h-7 px-2" @click="toggleAll(true)" title="Select all files">
        <CheckCircle2 :size="13" />
        All
      </button>
      <button v-if="!props.reviewMode" class="gl-command-btn h-7 px-2" @click="toggleAll(false)" title="Select no files">
        <CircleOff :size="13" />
        None
      </button>
    </div>
    <ul class="flex-1 overflow-auto px-2 flex flex-col gap-0.5">
      <li v-for="f in files" :key="f.path"
          class="gl-row group"
          :class="{ 'is-selected': isDiffSelected(f) }"
          @click="openDiff(f)">
        <input v-if="!props.reviewMode" type="checkbox" :checked="selected[f.path]" @click.stop
               @change="selected[f.path] = !selected[f.path]"
               class="w-3.5 h-3.5 rounded gl-checkbox" />
        <span class="inline-flex items-center justify-center w-5 h-5 rounded text-[11px] font-bold gl-mono"
              :style="{ background: statusOf(f.status).bg, color: statusOf(f.status).color }">
          {{ statusOf(f.status).letter }}
        </span>
        <span class="truncate flex-1 text-[13px] gl-selectable" style="color: var(--fg-2)">{{ f.path }}</span>
      </li>
      <li v-if="files.length === 0" class="gl-empty">
        <CheckCircle2 :size="24" style="color: var(--success)" />
        <span class="text-[13px]">Working tree clean</span>
      </li>
    </ul>
    <div v-if="!props.reviewMode" class="p-3 flex flex-col gap-2" style="border-top: 1px solid var(--border-soft); background: var(--bg)">
      <textarea v-model="message" rows="3" placeholder="Commit message…  (⌘K to focus)"
                class="gl-input resize-none gl-mono text-[13px]" />
      <div class="flex items-center justify-between gap-2">
        <label class="flex items-center gap-1.5 text-[12px]" style="color: var(--fg-2)">
          <input type="checkbox" v-model="skipHooks" class="w-3.5 h-3.5 rounded gl-checkbox" />
          Skip hooks (-n)
        </label>
        <button class="gl-command-btn gl-btn-primary"
                :disabled="checkedFiles.length === 0 || !message.trim()"
                @click="doCommit">
          <GitCommitHorizontal :size="14" />
          Commit {{ checkedFiles.length ? `(${checkedFiles.length})` : "" }}
        </button>
      </div>
    </div>
  </div>
</template>
