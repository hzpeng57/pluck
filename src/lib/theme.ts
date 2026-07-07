import { computed, ref, watch } from "vue";

export type ThemeMode = "system" | "light" | "dark";
export type ResolvedTheme = "light" | "dark";

const STORAGE_KEY = "pluck:themeMode";
const VALID_MODES = new Set<ThemeMode>(["system", "light", "dark"]);

function readStoredMode(): ThemeMode {
  try {
    const raw = localStorage.getItem(STORAGE_KEY) as ThemeMode | null;
    return raw && VALID_MODES.has(raw) ? raw : "system";
  } catch {
    return "system";
  }
}

export const themeMode = ref<ThemeMode>(readStoredMode());
const systemDark = ref(false);
let initialized = false;

export const resolvedTheme = computed<ResolvedTheme>(() => {
  if (themeMode.value === "system") return systemDark.value ? "dark" : "light";
  return themeMode.value;
});

function applyTheme() {
  if (typeof document === "undefined") return;
  document.documentElement.dataset.theme = resolvedTheme.value;
  document.documentElement.dataset.themeMode = themeMode.value;
  document.documentElement.style.colorScheme = resolvedTheme.value;
}

export function initTheme() {
  if (initialized || typeof window === "undefined") return;
  initialized = true;

  const media = window.matchMedia("(prefers-color-scheme: dark)");
  systemDark.value = media.matches;
  const onSystemChange = (event: MediaQueryListEvent) => {
    systemDark.value = event.matches;
  };

  if (typeof media.addEventListener === "function") {
    media.addEventListener("change", onSystemChange);
  } else {
    media.addListener(onSystemChange);
  }

  watch(themeMode, mode => {
    try { localStorage.setItem(STORAGE_KEY, mode); } catch {}
    applyTheme();
  }, { immediate: true });
  watch(resolvedTheme, applyTheme);
}

export function setThemeMode(mode: ThemeMode) {
  themeMode.value = mode;
}

export function useThemePreference() {
  initTheme();
  return { themeMode, resolvedTheme, setThemeMode };
}
