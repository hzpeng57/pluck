import { invoke } from "@tauri-apps/api/core";
import type { RepoMeta, RepoSnapshot, CommitDetail } from "../types/git";

export const api = {
  repoAdd: (path: string) => invoke<RepoMeta>("repo_add", { path }),
  repoOpen: (id: string) => invoke<RepoSnapshot>("repo_open", { id }),
  repoRefresh: (id: string, logBranch?: string) =>
    invoke<RepoSnapshot>("repo_refresh", { id, logBranch: logBranch ?? null }),
  commitDetail: (id: string, hash: string) =>
    invoke<CommitDetail>("commit_detail", { id, hash }),
};

export const ops = {
  branchCheckout: (id: string, name: string) =>
    invoke<RepoSnapshot>("branch_checkout", { id, name }),
  branchCreate: (id: string, name: string, from: string | null) =>
    invoke<RepoSnapshot>("branch_create", { id, name, from }),
  branchDelete: (id: string, name: string, force: boolean) =>
    invoke<RepoSnapshot>("branch_delete", { id, name, force }),
  commit: (id: string, files: string[], message: string, skipHooks: boolean) =>
    invoke<RepoSnapshot>("commit", { id, files, message, skipHooks }),
  merge: (id: string, branch: string) => invoke<RepoSnapshot>("merge", { id, branch }),
  mergeAbort: (id: string) => invoke<RepoSnapshot>("merge_abort_cmd", { id }),
  mergeContinue: (id: string) => invoke<RepoSnapshot>("merge_continue_cmd", { id }),
};
