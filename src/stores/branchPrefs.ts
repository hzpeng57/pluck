import { defineStore } from "pinia";
import { ref, watch } from "vue";

const LS_KEY = "pluck:branchPrefs";

interface Prefs { pinned: string[]; collapsed: string[] }
type AllPrefs = Record<string, Prefs>;

function load(): AllPrefs {
  try { return JSON.parse(localStorage.getItem(LS_KEY) ?? "{}"); } catch { return {}; }
}

export const useBranchPrefsStore = defineStore("branchPrefs", () => {
  const all = ref<AllPrefs>(load());
  watch(all, v => localStorage.setItem(LS_KEY, JSON.stringify(v)), { deep: true });

  function ensure(repoId: string): Prefs {
    if (!all.value[repoId]) all.value[repoId] = { pinned: [], collapsed: [] };
    return all.value[repoId];
  }
  function pinned(repoId: string): string[] { return ensure(repoId).pinned; }
  function isPinned(repoId: string, name: string): boolean { return ensure(repoId).pinned.includes(name); }
  function togglePin(repoId: string, name: string) {
    const p = ensure(repoId);
    const i = p.pinned.indexOf(name);
    if (i >= 0) p.pinned.splice(i, 1); else p.pinned.push(name);
  }
  function isCollapsed(repoId: string, prefix: string): boolean { return ensure(repoId).collapsed.includes(prefix); }
  function toggleCollapse(repoId: string, prefix: string) {
    const p = ensure(repoId);
    const i = p.collapsed.indexOf(prefix);
    if (i >= 0) p.collapsed.splice(i, 1); else p.collapsed.push(prefix);
  }

  return { all, pinned, isPinned, togglePin, isCollapsed, toggleCollapse };
});
