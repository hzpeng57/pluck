import { defineStore } from "pinia";
import { ref, computed, watch } from "vue";
import { api } from "../api/tauri";
import type { RepoSnapshot, CommitDetail, Commit, WorkingFile, ChangedFile, FileDiff, DiffTarget } from "../types/git";

const LOG_PAGE_SIZE = 200;
const DIFF_IGNORE_WS_KEY = "pluck:diffIgnoreWhitespace";

interface Toast { id: number; level: "error" | "info"; msg: string; loading?: boolean }
interface ConfirmOptions {
  title: string;
  message: string;
  confirmLabel?: string;
  cancelLabel?: string;
  tone?: "warning" | "danger";
  confirmText?: string;
}
interface ConfirmDialog extends ConfirmOptions {
  id: number;
  resolve: (ok: boolean) => void;
}

export const useRepoStateStore = defineStore("repoState", () => {
  const snapshot = ref<RepoSnapshot | null>(null);
  const loading = ref(false);
  const toasts = ref<Toast[]>([]);
  const selectedLogBranch = ref<string | null>(null);
  const selectedCommit = ref<CommitDetail | null>(null);
  const loadingCommit = ref(false);
  const diffTarget = ref<DiffTarget | null>(null);
  const selectedDiff = ref<FileDiff | null>(null);
  const loadingDiff = ref(false);
  const diffError = ref<string | null>(null);
  const diffIgnoreWhitespace = ref(localStorage.getItem(DIFF_IGNORE_WS_KEY) === "1");
  watch(diffIgnoreWhitespace, value => localStorage.setItem(DIFF_IGNORE_WS_KEY, value ? "1" : "0"));
  const diffOptions = computed(() => ({ ignoreWhitespace: diffIgnoreWhitespace.value }));
  const selectedHashes = ref<Set<string>>(new Set());
  const anchorHash = ref<string | null>(null);
  const selectionCount = computed(() => selectedHashes.value.size);
  const editMessageDialog = ref<{ hash: string; initial: string; mode: "amend" | "reword" } | null>(null);
  const resetDialog = ref<{ hash: string; short: string; subject: string } | null>(null);
  const branchCreateDialog = ref<{ from: string } | null>(null);
  const branchRenameDialog = ref<{ name: string; upstream: string | null } | null>(null);
  const branchDeleteDialog = ref<{ name: string } | null>(null);
  const confirmDialog = ref<ConfirmDialog | null>(null);
  const logEnd = ref(false);
  const logLoadingMore = ref(false);
  let nextId = 1;
  let activeRepoId: string | null = null;
  let snapshotRequestId = 0;
  let commitRequestId = 0;
  let diffRequestId = 0;
  const toastTimers = new Map<number, number>();

  // 每次 snapshot 整体替换（首次 open / refresh / 任何 mutation 后回流），
  // 重置 log 分页游标。追加 (snapshot.log = [...]) 不会触发 ref 变化。
  watch(snapshot, () => {
    logEnd.value = false;
    logLoadingMore.value = false;
    const snap = snapshot.value;
    if (!snap) {
      selectedLogBranch.value = null;
      return;
    }
    reconcileWorkingReview(snap);
    const selected = selectedLogBranch.value;
    const branches = [...snap.branches.local, ...snap.branches.remote];
    if (selected && !branches.some(b => b.name === selected)) {
      selectedLogBranch.value = snap.head.branch;
    }
  });

  function openEditMessageDialog(hash: string, initial: string, mode: "amend" | "reword") {
    editMessageDialog.value = { hash, initial, mode };
  }
  function closeEditMessageDialog() { editMessageDialog.value = null; }
  function openResetDialog(hash: string, short: string, subject: string) {
    resetDialog.value = { hash, short, subject };
  }
  function closeResetDialog() { resetDialog.value = null; }
  function openBranchCreateDialog(from: string) { branchCreateDialog.value = { from }; }
  function closeBranchCreateDialog() { branchCreateDialog.value = null; }
  function openBranchRenameDialog(name: string, upstream: string | null) { branchRenameDialog.value = { name, upstream }; }
  function closeBranchRenameDialog() { branchRenameDialog.value = null; }
  function openBranchDeleteDialog(name: string) { branchDeleteDialog.value = { name }; }
  function closeBranchDeleteDialog() { branchDeleteDialog.value = null; }
  function confirmAction(options: ConfirmOptions): Promise<boolean> {
    return new Promise(resolve => {
      confirmDialog.value = { id: nextId++, tone: "warning", ...options, resolve };
    });
  }
  function resolveConfirm(ok: boolean) {
    const dialog = confirmDialog.value;
    confirmDialog.value = null;
    dialog?.resolve(ok);
  }

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
    if (next.size !== 1) { clearCommitDetail(); }
    else { const only = [...next][0]; if (anchorHash.value === null) anchorHash.value = only; }
  }
  function selectRange(log: Commit[], hash: string) {
    if (!anchorHash.value) { selectedHashes.value = new Set([hash]); anchorHash.value = hash; clearCommitDetail(); return; }
    const a = log.findIndex(c => c.hash === anchorHash.value);
    const b = log.findIndex(c => c.hash === hash);
    if (a < 0 || b < 0) { selectedHashes.value = new Set([hash]); anchorHash.value = hash; clearCommitDetail(); return; }
    const [lo, hi] = a < b ? [a, b] : [b, a];
    selectedHashes.value = new Set(log.slice(lo, hi + 1).map(c => c.hash));
    clearCommitDetail();
  }
  function clearCommitDetail() {
    commitRequestId++;
    selectedCommit.value = null;
    loadingCommit.value = false;
  }
  function clearSelectionState() {
    selectedHashes.value = new Set();
    anchorHash.value = null;
    clearCommitDetail();
  }

  function clearSelection() {
    clearSelectionState();
  }

  function closeReviewMode() {
    diffRequestId++;
    diffTarget.value = null;
    selectedDiff.value = null;
    loadingDiff.value = false;
    diffError.value = null;
  }

  function sameWorkingTarget(target: DiffTarget, file: WorkingFile) {
    return target.kind === "workingTree"
      && target.path === file.path
      && target.oldPath === file.oldPath
      && target.status === file.status;
  }

  function reconcileWorkingReview(snap: RepoSnapshot) {
    const target = diffTarget.value;
    if (!target || target.kind !== "workingTree" || !activeRepoId) return;
    const freshFile = snap.files.find(file => file.path === target.path);
    if (!freshFile || !sameWorkingTarget(target, freshFile)) {
      closeReviewMode();
      return;
    }
    void openWorkingDiff(activeRepoId, freshFile);
  }

  function clearRepoView() {
    dismissLoadingToasts();
    snapshot.value = null;
    selectedLogBranch.value = null;
    clearSelectionState();
    closeReviewMode();
    editMessageDialog.value = null;
    resetDialog.value = null;
    branchCreateDialog.value = null;
    branchRenameDialog.value = null;
    branchDeleteDialog.value = null;
    logEnd.value = false;
    logLoadingMore.value = false;
  }

  function isCurrentSnapshotRequest(repoId: string, requestId: number) {
    return activeRepoId === repoId && snapshotRequestId === requestId;
  }

  async function selectCommit(repoId: string, hash: string) {
    if (activeRepoId !== repoId) return;
    const requestId = ++commitRequestId;
    loadingCommit.value = true;
    try {
      const detail = await api.commitDetail(repoId, hash);
      if (activeRepoId !== repoId || commitRequestId !== requestId) return;
      selectedCommit.value = detail;
    }
    catch (e: any) {
      if (activeRepoId === repoId && commitRequestId === requestId) pushToast("error", formatErr(e));
    }
    finally {
      if (activeRepoId === repoId && commitRequestId === requestId) loadingCommit.value = false;
    }
  }
  function clearSelectedCommit() {
    clearSelectionState();
  }

  async function openWorkingDiff(repoId: string, file: WorkingFile) {
    if (activeRepoId !== repoId) return;
    const requestId = ++diffRequestId;
    diffTarget.value = { kind: "workingTree", path: file.path, oldPath: file.oldPath, status: file.status };
    selectedDiff.value = null;
    diffError.value = null;
    loadingDiff.value = true;
    try {
      const diff = await api.workingFileDiff(repoId, file.path, file.oldPath, file.status, diffOptions.value);
      if (activeRepoId !== repoId || diffRequestId !== requestId) return;
      selectedDiff.value = diff;
    }
    catch (e: any) {
      if (activeRepoId === repoId && diffRequestId === requestId) diffError.value = formatErr(e);
    }
    finally {
      if (activeRepoId === repoId && diffRequestId === requestId) loadingDiff.value = false;
    }
  }

  async function openCommitFileDiff(repoId: string, commit: CommitDetail, file: ChangedFile) {
    if (activeRepoId !== repoId) return;
    const requestId = ++diffRequestId;
    diffTarget.value = { kind: "commit", hash: commit.hash, path: file.path, oldPath: file.oldPath, status: file.status };
    selectedDiff.value = null;
    diffError.value = null;
    loadingDiff.value = true;
    try {
      const diff = await api.commitFileDiff(repoId, commit.hash, file.path, file.oldPath, file.status, diffOptions.value);
      if (activeRepoId !== repoId || diffRequestId !== requestId) return;
      selectedDiff.value = diff;
    }
    catch (e: any) {
      if (activeRepoId === repoId && diffRequestId === requestId) diffError.value = formatErr(e);
    }
    finally {
      if (activeRepoId === repoId && diffRequestId === requestId) loadingDiff.value = false;
    }
  }

  async function rollbackCurrentWorkingFile(repoId: string) {
    if (activeRepoId !== repoId) return;
    const target = diffTarget.value;
    if (!target || target.kind !== "workingTree") return;
    snapshotRequestId++;
    loading.value = true;
    diffError.value = null;
    try {
      const next = await api.rollbackFile(repoId, target.path, target.oldPath, target.status);
      if (activeRepoId !== repoId) return;
      snapshotRequestId++;
      snapshot.value = next;
      closeReviewMode();
    }
    catch (e: any) {
      if (activeRepoId === repoId) diffError.value = formatErr(e);
    }
    finally {
      if (activeRepoId === repoId) loading.value = false;
    }
  }

  async function reloadCurrentDiff(repoId: string) {
    const target = diffTarget.value;
    if (!target) return;
    if (target.kind === "workingTree") {
      await openWorkingDiff(repoId, { path: target.path, oldPath: target.oldPath, status: target.status });
      return;
    }
    if (selectedCommit.value) {
      await openCommitFileDiff(repoId, selectedCommit.value, {
        path: target.path,
        oldPath: target.oldPath,
        status: target.status,
      });
    }
  }

  async function setDiffIgnoreWhitespace(repoId: string, value: boolean) {
    if (diffIgnoreWhitespace.value === value) return;
    diffIgnoreWhitespace.value = value;
    await reloadCurrentDiff(repoId);
  }

  function dismissToast(id: number) {
    const timer = toastTimers.get(id);
    if (timer !== undefined) {
      window.clearTimeout(timer);
      toastTimers.delete(id);
    }
    toasts.value = toasts.value.filter(t => t.id !== id);
  }

  function dismissLoadingToasts() {
    for (const id of toasts.value.filter(toast => toast.loading).map(toast => toast.id)) dismissToast(id);
  }

  function pushToast(level: "error" | "info", msg: string, options?: { loading?: boolean; durationMs?: number }) {
    const id = nextId++;
    toasts.value.push({ id, level, msg, loading: options?.loading });
    if (!options?.loading) {
      const durationMs = options?.durationMs ?? 6000;
      const timer = window.setTimeout(() => { dismissToast(id); }, durationMs);
      toastTimers.set(id, timer);
    }
    return id;
  }

  function pushLoadingToast(msg: string) {
    return pushToast("info", msg, { loading: true });
  }

  async function open(id: string) {
    activeRepoId = id;
    const requestId = ++snapshotRequestId;
    clearRepoView();
    loading.value = true;
    try {
      const next = await api.repoOpen(id);
      if (!isCurrentSnapshotRequest(id, requestId)) return;
      snapshot.value = next;
      selectedLogBranch.value = next.head.branch;
    }
    catch (e: any) {
      if (isCurrentSnapshotRequest(id, requestId)) pushToast("error", formatErr(e));
    }
    finally {
      if (isCurrentSnapshotRequest(id, requestId)) loading.value = false;
    }
  }
  async function refresh(id: string) {
    if (activeRepoId !== id) return;
    const requestId = ++snapshotRequestId;
    loading.value = true;
    try {
      const next = await api.repoRefresh(id, selectedLogBranch.value ?? undefined);
      if (!isCurrentSnapshotRequest(id, requestId)) return;
      snapshot.value = next;
    }
    catch (e: any) {
      if (isCurrentSnapshotRequest(id, requestId)) pushToast("error", formatErr(e));
    }
    finally {
      if (isCurrentSnapshotRequest(id, requestId)) loading.value = false;
    }
  }
  function setLogBranch(id: string, branch: string) { selectedLogBranch.value = branch; refresh(id); }

  async function loadMoreLog(id: string) {
    if (logLoadingMore.value || logEnd.value || !snapshot.value) return;
    const requestId = snapshotRequestId;
    logLoadingMore.value = true;
    try {
      const branch = selectedLogBranch.value;
      const skip = snapshot.value.log.length;
      const next = await api.logPage(id, branch, skip, LOG_PAGE_SIZE);
      if (!isCurrentSnapshotRequest(id, requestId) || !snapshot.value) return;
      if (next.length === 0) { logEnd.value = true; return; }
      // 追加而不是替换 snapshot，避免 watcher 重置游标
      snapshot.value.log = [...snapshot.value.log, ...next];
      if (next.length < LOG_PAGE_SIZE) logEnd.value = true;
    } catch (e: any) {
      if (isCurrentSnapshotRequest(id, requestId)) pushToast("error", formatErr(e));
    } finally {
      if (isCurrentSnapshotRequest(id, requestId)) logLoadingMore.value = false;
    }
  }

  return {
    snapshot, loading, toasts, selectedLogBranch, selectedCommit, loadingCommit,
    diffTarget, selectedDiff, loadingDiff, diffError, diffIgnoreWhitespace,
    selectedHashes, anchorHash, selectionCount,
    editMessageDialog, resetDialog, branchCreateDialog, branchRenameDialog, branchDeleteDialog,
    confirmDialog,
    logEnd, logLoadingMore,
    open, refresh, setLogBranch, pushToast, pushLoadingToast, dismissToast,
    selectCommit, clearSelectedCommit,
    openWorkingDiff, openCommitFileDiff, closeReviewMode, rollbackCurrentWorkingFile, setDiffIgnoreWhitespace,
    setSingleSelection, toggleSelection, selectRange, clearSelection,
    openEditMessageDialog, closeEditMessageDialog, openResetDialog, closeResetDialog,
    openBranchCreateDialog, closeBranchCreateDialog,
    openBranchRenameDialog, closeBranchRenameDialog,
    openBranchDeleteDialog, closeBranchDeleteDialog,
    confirmAction, resolveConfirm,
    loadMoreLog,
  };
});

function formatErr(e: any): string {
  if (e?.kind === "GitExit") return e.data.friendly;
  if (typeof e === "string") return e;
  return e?.message ?? JSON.stringify(e);
}
