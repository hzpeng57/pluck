import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { api } from "../api/tauri";
import type { RepoSnapshot, CommitDetail, Commit } from "../types/git";

interface Toast { id: number; level: "error" | "info"; msg: string }

export const useRepoStateStore = defineStore("repoState", () => {
  const snapshot = ref<RepoSnapshot | null>(null);
  const loading = ref(false);
  const toasts = ref<Toast[]>([]);
  const selectedLogBranch = ref<string | null>(null);
  const selectedCommit = ref<CommitDetail | null>(null);
  const loadingCommit = ref(false);
  const selectedHashes = ref<Set<string>>(new Set());
  const anchorHash = ref<string | null>(null);
  const selectionCount = computed(() => selectedHashes.value.size);
  const editMessageDialog = ref<{ hash: string; initial: string; mode: "amend" | "reword" } | null>(null);
  const resetDialog = ref<{ hash: string; short: string; subject: string } | null>(null);
  let nextId = 1;

  function openEditMessageDialog(hash: string, initial: string, mode: "amend" | "reword") {
    editMessageDialog.value = { hash, initial, mode };
  }
  function closeEditMessageDialog() { editMessageDialog.value = null; }
  function openResetDialog(hash: string, short: string, subject: string) {
    resetDialog.value = { hash, short, subject };
  }
  function closeResetDialog() { resetDialog.value = null; }

  function setSingleSelection(repoId: string, hash: string) {
    selectedHashes.value = new Set([hash]);
    anchorHash.value = hash;
    void selectCommit(repoId, hash);
  }
  function toggleSelection(hash: string) {
    const next = new Set(selectedHashes.value);
    if (next.has(hash)) next.delete(hash);
    else next.add(hash);
    selectedHashes.value = next;
    if (next.size !== 1) { selectedCommit.value = null; }
    else { const only = [...next][0]; if (anchorHash.value === null) anchorHash.value = only; }
  }
  function selectRange(log: Commit[], hash: string) {
    if (!anchorHash.value) { selectedHashes.value = new Set([hash]); anchorHash.value = hash; selectedCommit.value = null; return; }
    const a = log.findIndex(c => c.hash === anchorHash.value);
    const b = log.findIndex(c => c.hash === hash);
    if (a < 0 || b < 0) { selectedHashes.value = new Set([hash]); anchorHash.value = hash; selectedCommit.value = null; return; }
    const [lo, hi] = a < b ? [a, b] : [b, a];
    selectedHashes.value = new Set(log.slice(lo, hi + 1).map(c => c.hash));
    selectedCommit.value = null;
  }
  function clearSelection() {
    selectedHashes.value = new Set();
    anchorHash.value = null;
    selectedCommit.value = null;
  }

  async function selectCommit(repoId: string, hash: string) {
    loadingCommit.value = true;
    try { selectedCommit.value = await api.commitDetail(repoId, hash); }
    catch (e: any) { pushToast("error", formatErr(e)); }
    finally { loadingCommit.value = false; }
  }
  function clearSelectedCommit() {
    selectedCommit.value = null;
    selectedHashes.value = new Set();
    anchorHash.value = null;
  }

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

  return {
    snapshot, loading, toasts, selectedLogBranch, selectedCommit, loadingCommit,
    selectedHashes, anchorHash, selectionCount,
    editMessageDialog, resetDialog,
    open, refresh, setLogBranch, pushToast, selectCommit, clearSelectedCommit,
    setSingleSelection, toggleSelection, selectRange, clearSelection,
    openEditMessageDialog, closeEditMessageDialog, openResetDialog, closeResetDialog,
  };
});

function formatErr(e: any): string {
  if (e?.kind === "GitExit") return e.data.friendly;
  if (typeof e === "string") return e;
  return e?.message ?? JSON.stringify(e);
}
