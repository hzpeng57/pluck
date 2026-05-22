import { defineStore } from "pinia";
import { ref } from "vue";
import { api } from "../api/tauri";
import type { RepoSnapshot } from "../types/git";

interface Toast { id: number; level: "error" | "info"; msg: string }

export const useRepoStateStore = defineStore("repoState", () => {
  const snapshot = ref<RepoSnapshot | null>(null);
  const loading = ref(false);
  const toasts = ref<Toast[]>([]);
  const selectedLogBranch = ref<string | null>(null);
  let nextId = 1;

  function pushToast(level: "error" | "info", msg: string) {
    const id = nextId++;
    toasts.value.push({ id, level, msg });
    setTimeout(() => { toasts.value = toasts.value.filter(t => t.id !== id); }, 6000);
  }

  async function open(id: string) {
    loading.value = true;
    try { snapshot.value = await api.repoOpen(id); selectedLogBranch.value = snapshot.value.head.branch; }
    catch (e: any) { pushToast("error", formatErr(e)); }
    finally { loading.value = false; }
  }
  async function refresh(id: string) {
    loading.value = true;
    try { snapshot.value = await api.repoRefresh(id, selectedLogBranch.value ?? undefined); }
    catch (e: any) { pushToast("error", formatErr(e)); }
    finally { loading.value = false; }
  }
  function setLogBranch(id: string, branch: string) { selectedLogBranch.value = branch; refresh(id); }

  return { snapshot, loading, toasts, selectedLogBranch, open, refresh, setLogBranch, pushToast };
});

function formatErr(e: any): string {
  if (e?.kind === "GitExit") return e.data.friendly;
  if (typeof e === "string") return e;
  return e?.message ?? JSON.stringify(e);
}
