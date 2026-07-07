<script setup lang="ts">
import { computed } from "vue";
import { X } from "lucide-vue-next";
import { updaterState, checkForUpdates, downloadAndInstall, restart, dismiss } from "../lib/updater";

const visible = computed(() => {
  const k = updaterState.value.kind;
  return k === "available" || k === "downloading" || k === "ready" || k === "error";
});

const label = computed(() => {
  const s = updaterState.value;
  switch (s.kind) {
    case "available": return `Pluck ${s.version} is available`;
    case "downloading": return `Downloading… ${Math.round(s.percent * 100)}%`;
    case "ready": return `Pluck ${s.version} ready — restart to apply`;
    case "error": return `Update check failed: ${s.message}`;
    default: return "";
  }
});

async function onPrimary() {
  const s = updaterState.value;
  if (s.kind === "available") await downloadAndInstall();
  else if (s.kind === "ready") await restart();
  else if (s.kind === "error") await checkForUpdates(true);
}

const primaryLabel = computed(() => {
  const s = updaterState.value;
  switch (s.kind) {
    case "available": return "Download & Install";
    case "ready": return "Restart Now";
    case "error": return "Retry";
    default: return null;
  }
});
</script>

<template>
  <div v-if="visible" class="gl-banner px-3 py-2 text-[13px]"
       :style="{
         background: updaterState.kind === 'error' ? 'var(--danger-soft-weak)' : 'var(--raised)',
         borderColor: updaterState.kind === 'error' ? 'var(--danger-ring)' : 'var(--border-soft)'
       }">
    <span class="w-1.5 h-1.5 rounded-full shrink-0"
          :style="{ background: updaterState.kind === 'error' ? 'var(--danger)' : 'var(--info)' }" />
    <span class="flex-1 truncate">{{ label }}</span>
    <button v-if="primaryLabel" class="gl-command-btn gl-btn-primary" @click="onPrimary">{{ primaryLabel }}</button>
    <button class="gl-icon-btn" title="Dismiss" @click="dismiss">
      <X :size="14" />
    </button>
  </div>
</template>
