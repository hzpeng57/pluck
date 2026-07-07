<script setup lang="ts">
import { computed } from "vue";
import {
  Download,
  Monitor,
  Moon,
  RefreshCw,
  RotateCcw,
  Settings,
  Sun,
  X,
} from "lucide-vue-next";
import { version } from "../../package.json";
import { useRepoStateStore } from "../stores/repoState";
import { downloadAndInstall, restart, updaterState, checkForUpdates } from "../lib/updater";
import { setThemeMode, useThemePreference, type ThemeMode } from "../lib/theme";

const state = useRepoStateStore();
const { themeMode } = useThemePreference();
const { open = false } = defineProps<{ open?: boolean }>();
const emit = defineEmits<{ close: [] }>();

const themeOptions: { value: ThemeMode; label: string; icon: typeof Monitor }[] = [
  { value: "system", label: "System", icon: Monitor },
  { value: "light", label: "Light", icon: Sun },
  { value: "dark", label: "Dark", icon: Moon },
];

const activeThemeIndex = computed(() =>
  Math.max(0, themeOptions.findIndex(option => option.value === themeMode.value))
);

const updateLabel = computed(() => {
  const current = updaterState.value;
  switch (current.kind) {
    case "checking": return "Checking for updates...";
    case "uptodate": return "Pluck is up to date";
    case "available": return `Pluck ${current.version} is available`;
    case "downloading": return `Downloading ${Math.round(current.percent * 100)}%`;
    case "ready": return `Pluck ${current.version} is ready to install`;
    case "error": return `Update check failed: ${current.message}`;
    default: return "Check whether a newer Pluck build is available.";
  }
});

const updateButton = computed(() => {
  const current = updaterState.value;
  switch (current.kind) {
    case "checking":
    case "downloading":
      return { label: current.kind === "checking" ? "Checking" : "Downloading", icon: RefreshCw, disabled: true };
    case "available":
      return { label: "Download & Install", icon: Download, disabled: false };
    case "ready":
      return { label: "Restart Now", icon: RotateCcw, disabled: false };
    case "error":
      return { label: "Retry", icon: RefreshCw, disabled: false };
    default:
      return { label: "Check for Updates", icon: RefreshCw, disabled: false };
  }
});

function close() {
  emit("close");
}

function chooseTheme(mode: ThemeMode) {
  setThemeMode(mode);
}

async function onUpdateAction() {
  const current = updaterState.value;
  if (current.kind === "checking" || current.kind === "downloading") return;
  if (current.kind === "available") {
    await downloadAndInstall();
    return;
  }
  if (current.kind === "ready") {
    await restart();
    return;
  }
  await checkForUpdates(true);
  if (updaterState.value.kind === "uptodate") {
    state.pushToast("info", `You are on the latest version (v${version})`);
  }
}

function onKey(e: KeyboardEvent) {
  if (e.key === "Escape") {
    e.preventDefault();
    close();
  }
}
</script>

<template>
  <div v-if="open"
       class="gl-overlay fixed inset-0 flex items-center justify-center z-50"
       @click.self="close"
       @keydown="onKey">
    <div class="gl-dialog-shell w-[520px] flex flex-col">
      <div class="gl-panel-header h-12 shrink-0">
        <Settings :size="16" style="color: var(--accent)" />
        <span class="font-semibold text-[13.5px]">Settings</span>
        <div class="flex-1" />
        <button class="gl-icon-btn" title="Close" @click="close">
          <X :size="14" />
        </button>
      </div>

      <div class="p-4 flex flex-col gap-4">
        <section class="flex flex-col gap-2">
          <div class="flex items-center justify-between gap-3">
            <div>
              <div class="font-medium text-[13.5px]" style="color: var(--fg)">Appearance</div>
            </div>
          </div>

          <div class="gl-theme-slider" role="radiogroup" aria-label="Theme mode">
            <span class="gl-theme-slider-thumb"
                  :style="{ transform: `translateX(${activeThemeIndex * 100}%)` }" />
            <button v-for="option in themeOptions"
                    :key="option.value"
                    type="button"
                    class="gl-theme-slider-option"
                    role="radio"
                    :aria-checked="themeMode === option.value"
                    :class="{ 'is-active': themeMode === option.value }"
                    @click="chooseTheme(option.value)">
              <component :is="option.icon" :size="15" />
              <span>{{ option.label }}</span>
            </button>
          </div>
        </section>

        <section class="flex flex-col gap-3 pt-1">
          <div>
            <div class="font-medium text-[13.5px]" style="color: var(--fg)">Updates</div>
            <div class="text-[12px]" style="color: var(--fg-3)">{{ updateLabel }}</div>
          </div>
          <div class="flex items-center gap-2">
            <span class="gl-badge">pluck v{{ version }}</span>
            <div class="flex-1" />
            <button class="gl-command-btn"
                    :disabled="updateButton.disabled"
                    @click="onUpdateAction">
              <component :is="updateButton.icon"
                         :size="14"
                         :class="{ 'animate-spin': updaterState.kind === 'checking' || updaterState.kind === 'downloading' }" />
              {{ updateButton.label }}
            </button>
          </div>
        </section>
      </div>
    </div>
  </div>
</template>
