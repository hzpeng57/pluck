import { invoke } from "@tauri-apps/api/core";
import type { RepoMeta, RepoSnapshot, CommitDetail, DeletePrecheck, Commit } from "../types/git";

export const api = {
  repoAdd: (path: string) => invoke<RepoMeta>("repo_add", { path }),
  repoOpen: (id: string) => invoke<RepoSnapshot>("repo_open", { id }),
  repoRefresh: (id: string, logBranch?: string) =>
    invoke<RepoSnapshot>("repo_refresh", { id, logBranch: logBranch ?? null }),
  logPage: (id: string, branch: string | null, skip: number, limit: number) =>
    invoke<Commit[]>("log_page_cmd", { id, branch, skip, limit }),
  logSearch: (id: string, branch: string | null, query: string, author: string, limit: number) =>
    invoke<Commit[]>("log_search_cmd", { id, branch, query, author, limit }),
  commitDetail: (id: string, hash: string) =>
    invoke<CommitDetail>("commit_detail", { id, hash }),
  cherryPick: (id: string, hashes: string[]) =>
    invoke<RepoSnapshot>("cherry_pick_cmd", { id, hashes }),
  revert: (id: string, hashes: string[]) =>
    invoke<RepoSnapshot>("revert_cmd", { id, hashes }),
  resetTo: (id: string, hash: string, mode: "soft" | "mixed" | "hard" | "keep") =>
    invoke<RepoSnapshot>("reset_to_commit", { id, hash, mode }),
  amendHeadMessage: (id: string, message: string) =>
    invoke<RepoSnapshot>("amend_head_message", { id, message }),
  rewordAncestor: (id: string, hash: string, message: string) =>
    invoke<RepoSnapshot>("reword_ancestor", { id, hash, message }),
};

export const ops = {
  branchCheckout: (id: string, name: string) =>
    invoke<RepoSnapshot>("branch_checkout", { id, name }),
  branchCreate: (id: string, name: string, from: string | null) =>
    invoke<RepoSnapshot>("branch_create", { id, name, from }),
  branchRename: (id: string, oldName: string, newName: string, unsetUpstream: boolean) =>
    invoke<RepoSnapshot>("branch_rename", { id, oldName, newName, unsetUpstream }),
  branchDelete: (id: string, name: string, force: boolean) =>
    invoke<RepoSnapshot>("branch_delete", { id, name, force }),
  branchDeletePrecheck: (id: string, name: string) =>
    invoke<DeletePrecheck>("branch_delete_precheck", { id, name }),
  commit: (id: string, files: string[], message: string, skipHooks: boolean) =>
    invoke<RepoSnapshot>("commit", { id, files, message, skipHooks }),
  merge: (id: string, branch: string) => invoke<RepoSnapshot>("merge", { id, branch }),
  mergeAbort: (id: string) => invoke<RepoSnapshot>("merge_abort_cmd", { id }),
  mergeContinue: (id: string) => invoke<RepoSnapshot>("merge_continue_cmd", { id }),
};
