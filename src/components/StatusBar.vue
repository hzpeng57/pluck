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
async function onCheckUpdate() {
  if (checking.value) return;
  await checkForUpdates(true);
  if (updaterState.value.kind === "uptodate") {
    state.pushToast("info", `You are on the latest version (v${version})`);
  }
}
</script>
<template>
  <footer class="flex items-center gap-2 px-3 h-7 shrink-0 text-[12px]"
          style="background: var(--bg); border-top: 1px solid var(--border-soft); color: var(--fg-3)">
    <span class="gl-badge">
      <span class="w-1.5 h-1.5 rounded-full"
            :style="{ background: counts.dirty ? 'var(--warning)' : 'var(--fg-3)' }" />
      <span class="gl-mono">{{ counts.dirty }}</span> dirty
    </span>
    <span class="gl-badge" :style="{ color: counts.ahead ? 'var(--success)' : 'var(--fg-3)' }">
      ↑ <span class="gl-mono">{{ counts.ahead }}</span>
    </span>
    <span class="gl-badge" :style="{ color: counts.behind ? 'var(--danger)' : 'var(--fg-3)' }">
      ↓ <span class="gl-mono">{{ counts.behind }}</span>
    </span>
    <div class="flex-1" />
    <span v-if="state.loading" class="flex items-center gap-1.5">
      <span class="gl-spinner" />
      refreshing
    </span>
    <button class="gl-badge hover:opacity-100 transition-opacity"
            :title="checking ? 'Checking for updates…' : 'Click to check for updates'"
            :disabled="checking"
            @click="onCheckUpdate">
      pluck v{{ version }}{{ checking ? ' · checking' : '' }}
    </button>
  </footer>
</template>
