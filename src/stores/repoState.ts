import { defineStore } from "pinia";
import { ref } from "vue";
import { api } from "../api/tauri";
import type { RepoSnapshot } from "../types/git";

export const useRepoStateStore = defineStore("repoState", () => {
  const snapshot = ref<RepoSnapshot | null>(null);
  const loading = ref(false);
  const lastError = ref<string | null>(null);
  const selectedLogBranch = ref<string | null>(null);

  async function open(id: string) {
    loading.value = true; lastError.value = null;
    try { snapshot.value = await api.repoOpen(id); selectedLogBranch.value = snapshot.value.head.branch; }
    catch (e: any) { lastError.value = formatErr(e); }
    finally { loading.value = false; }
  }
  async function refresh(id: string) {
    loading.value = true; lastError.value = null;
    try { snapshot.value = await api.repoRefresh(id, selectedLogBranch.value ?? undefined); }
    catch (e: any) { lastError.value = formatErr(e); }
    finally { loading.value = false; }
  }
  function setLogBranch(id: string, branch: string) { selectedLogBranch.value = branch; refresh(id); }

  return { snapshot, loading, lastError, selectedLogBranch, open, refresh, setLogBranch };
});

function formatErr(e: any): string {
  if (e?.kind === "GitExit") return e.data.friendly;
  if (typeof e === "string") return e;
  return e?.message ?? JSON.stringify(e);
}
