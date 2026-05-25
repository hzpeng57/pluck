<script setup lang="ts">
import { ref, computed } from "vue";
import { useRepoStateStore } from "../stores/repoState";
import type { ChangedFile, CommitDetail } from "../types/git";
import { buildFileTree, type FileTreeEntry } from "../lib/fileTree";

const state = useRepoStateStore();
const detail = computed<CommitDetail | null>(() => state.selectedCommit);

const collapsed = ref<Set<string>>(new Set());
function toggle(prefix: string) {
  const next = new Set(collapsed.value);
  if (next.has(prefix)) next.delete(prefix); else next.add(prefix);
  collapsed.value = next;
}
const tree = computed<FileTreeEntry[]>(() =>
  buildFileTree(detail.value?.files ?? [], p => collapsed.value.has(p))
);
const showAllRefs = ref(false);

function statusMeta(s: ChangedFile["status"]) {
  switch (s) {
    case "added":      return { letter: "A", color: "#34D399", bg: "rgba(52, 211, 153, 0.14)" };
    case "modified":   return { letter: "M", color: "#FBBF24", bg: "rgba(251, 191, 36, 0.14)" };
    case "deleted":    return { letter: "D", color: "#F87171", bg: "rgba(248, 113, 113, 0.14)" };
    case "renamed":    return { letter: "R", color: "#60A5FA", bg: "rgba(96, 165, 250, 0.14)" };
    case "copied":     return { letter: "C", color: "#A78BFA", bg: "rgba(167, 139, 250, 0.14)" };
    case "typechange": return { letter: "T", color: "#F472B6", bg: "rgba(244, 114, 182, 0.14)" };
  }
}

function formatAbsolute(unix: number): string {
  const d = new Date(unix * 1000);
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}/${d.getMonth() + 1}/${d.getDate()} at ${pad(d.getHours())}:${pad(d.getMinutes())}`;
}
</script>

<template>
  <div v-if="detail" class="flex flex-col h-full">
    <!-- Top bar: SHA + close -->
    <div class="flex items-center gap-2 px-3 pt-3 pb-2">
      <span class="gl-section-title">Commit</span>
      <span class="gl-mono text-[11px] px-1.5 py-0.5 rounded"
            style="background: var(--accent-soft); color: var(--accent-2)">{{ detail.short }}</span>
      <span class="gl-chip">{{ detail.files.length }} {{ detail.files.length === 1 ? "file" : "files" }}</span>
      <div class="flex-1" />
      <button class="gl-btn" @click="state.clearSelectedCommit()" title="Back to working tree (Esc)">
        ✕ Close
      </button>
    </div>

    <!-- File tree -->
    <ul class="flex-1 overflow-auto px-2 flex flex-col gap-0.5">
      <template v-for="entry in tree" :key="entry.kind === 'folder' ? 'd:' + entry.prefix : 'f:' + entry.file.path">
        <li v-if="entry.kind === 'folder'"
            @click="toggle(entry.prefix)"
            :title="entry.prefix"
            class="gl-row group">
          <span :style="{ paddingLeft: (entry.depth * 12) + 'px' }" class="inline-flex" />
          <span class="text-[10px] w-3 inline-flex justify-center transition-transform"
                style="color: var(--fg-3)"
                :style="{ transform: entry.collapsed ? 'rotate(0)' : 'rotate(90deg)' }">▶</span>
          <span class="text-[12px]" style="color: var(--accent-2)">▦</span>
          <span class="truncate flex-1 text-[13px]" style="color: var(--fg)">{{ entry.label }}</span>
          <span class="gl-chip">{{ entry.fileCount }}</span>
        </li>
        <li v-else
            :title="entry.file.oldPath ? `${entry.file.oldPath} → ${entry.file.path}` : entry.file.path"
            class="gl-row group" style="cursor: default">
          <span :style="{ paddingLeft: (entry.depth * 12) + 'px' }" class="inline-flex" />
          <span class="inline-flex items-center justify-center w-5 h-5 rounded text-[10px] font-bold gl-mono shrink-0"
                :style="{ background: statusMeta(entry.file.status).bg, color: statusMeta(entry.file.status).color }">
            {{ statusMeta(entry.file.status).letter }}
          </span>
          <span class="truncate flex-1 text-[12.5px] gl-mono gl-selectable" style="color: var(--fg-2)">
            {{ entry.displayLabel }}
          </span>
        </li>
      </template>
      <li v-if="detail.files.length === 0"
          class="flex flex-col items-center justify-center gap-1 py-8 text-center"
          style="color: var(--fg-3)">
        <span class="text-2xl">∅</span>
        <span class="text-[12px]">No file changes</span>
      </li>
    </ul>

    <!-- Metadata footer -->
    <div class="p-3 flex flex-col gap-1.5 shrink-0 gl-selectable"
         style="border-top: 1px solid var(--border-soft); background: var(--bg)">
      <div class="text-[13px] font-semibold gl-selectable" style="color: var(--fg)">{{ detail.subject }}</div>
      <pre v-if="detail.body.trim()"
           class="text-[12px] whitespace-pre-wrap gl-selectable gl-mono m-0"
           style="color: var(--fg-2)">{{ detail.body.trim() }}</pre>
      <div class="flex items-center gap-1.5 flex-wrap text-[11.5px]" style="color: var(--fg-2)">
        <span class="gl-mono px-1.5 py-0.5 rounded gl-selectable"
              style="background: var(--hover); color: var(--fg-3)">{{ detail.short }}</span>
        <span class="font-medium gl-selectable" style="color: var(--fg)">{{ detail.author }}</span>
        <span class="gl-selectable" style="color: var(--fg-3)">&lt;{{ detail.email }}&gt;</span>
        <span style="color: var(--fg-3)">·</span>
        <span class="gl-selectable" style="color: var(--fg-2)">{{ formatAbsolute(detail.dateUnix) }}</span>
      </div>
      <div v-if="detail.refs.length" class="flex items-center gap-1 flex-wrap text-[11px]">
        <span style="color: var(--fg-3)">In:</span>
        <template v-for="(r, i) in (showAllRefs ? detail.refs : detail.refs.slice(0, 3))" :key="r">
          <span class="gl-mono px-1.5 py-0.5 rounded gl-selectable"
                style="background: var(--accent-soft); color: var(--accent-2)">{{ r }}</span>
          <span v-if="i < (showAllRefs ? detail.refs.length : Math.min(3, detail.refs.length)) - 1"
                style="color: var(--fg-3)">·</span>
        </template>
        <button v-if="!showAllRefs && detail.refs.length > 3"
                class="text-[11px] underline"
                style="color: var(--accent-2)"
                @click="showAllRefs = true">
          Show all ({{ detail.refs.length }})
        </button>
      </div>
      <div v-if="detail.parents.length" class="text-[11px] flex gap-1.5 flex-wrap" style="color: var(--fg-3)">
        Parents:
        <span v-for="p in detail.parents" :key="p"
              class="gl-mono gl-selectable" style="color: var(--fg-2)">{{ p.slice(0, 7) }}</span>
      </div>
    </div>
  </div>
</template>
