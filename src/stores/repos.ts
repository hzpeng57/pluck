import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type { RepoMeta } from "../types/git";

const LS_KEY = "pluck:repos";

export const useReposStore = defineStore("repos", () => {
  const all = ref<RepoMeta[]>(loadFromLS());
  const activeId = ref<string | null>(all.value[0]?.id ?? null);
  const active = computed(() => all.value.find(r => r.id === activeId.value) ?? null);

  function add(meta: RepoMeta) {
    if (!all.value.find(r => r.id === meta.id)) all.value.push(meta);
    activeId.value = meta.id;
    persist();
  }
  function remove(id: string) {
    all.value = all.value.filter(r => r.id !== id);
    if (activeId.value === id) activeId.value = all.value[0]?.id ?? null;
    persist();
  }
  function setActive(id: string) { activeId.value = id; }
  function persist() { localStorage.setItem(LS_KEY, JSON.stringify(all.value)); }

  return { all, activeId, active, add, remove, setActive };
});

function loadFromLS(): RepoMeta[] {
  try { return JSON.parse(localStorage.getItem(LS_KEY) ?? "[]"); } catch { return [] }
}
