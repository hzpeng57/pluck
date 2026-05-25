<script setup lang="ts">
import { computed } from "vue";
import { useRepoStateStore } from "../stores/repoState";
import { version } from "../../package.json";
import { updaterState, checkForUpdates } from "../lib/updater";
const state = useRepoStateStore();
const counts = computed(() => {
  const s = state.snapshot; if (!s) return { dirty: 0, ahead: 0, behind: 0 };
  return { dirty: s.files.length, ahead: s.remoteStatus.ahead, behind: s.remoteStatus.behind };
});
const checking = computed(() => updaterState.value.kind === "checking");
function onCheckUpdate() { if (!checking.value) checkForUpdates(true); }
</script>
<template>
  <footer class="flex items-center gap-4 px-4 h-7 shrink-0 text-[11px]"
          style="background: var(--bg); border-top: 1px solid var(--border-soft); color: var(--fg-3)">
    <span class="flex items-center gap-1">
      <span class="w-1 h-1 rounded-full" :style="{ background: counts.dirty ? 'var(--warning)' : 'var(--fg-3)' }" />
      <span class="gl-mono">{{ counts.dirty }}</span> dirty
    </span>
    <span class="flex items-center gap-1">
      <span :style="{ color: counts.ahead ? 'var(--success)' : 'inherit' }">↑</span>
      <span class="gl-mono">{{ counts.ahead }}</span>
    </span>
    <span class="flex items-center gap-1">
      <span :style="{ color: counts.behind ? 'var(--danger)' : 'inherit' }">↓</span>
      <span class="gl-mono">{{ counts.behind }}</span>
    </span>
    <div class="flex-1" />
    <span v-if="state.loading" class="flex items-center gap-1.5">
      <span class="w-1.5 h-1.5 rounded-full animate-pulse" style="background: var(--accent)" />
      refreshing…
    </span>
    <button class="gl-mono opacity-60 hover:opacity-100 transition-opacity"
            :title="checking ? 'Checking for updates…' : 'Click to check for updates'"
            :disabled="checking"
            @click="onCheckUpdate">
      pluck v{{ version }}{{ checking ? ' · checking…' : '' }}
    </button>
  </footer>
</template>
