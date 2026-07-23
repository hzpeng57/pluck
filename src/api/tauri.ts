import { invoke } from "@tauri-apps/api/core";
import type {
  RepoMeta,
  RepoSnapshot,
  CommitDetail,
  DeletePrecheck,
  Commit,
  FileDiff,
  FileStatus,
  ChangedFileStatus,
  ConflictFile,
  ConflictFileDetail,
  ConflictStageChoice,
  DiffOptions,
  BranchKind,
} from "../types/git";

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
  workingFileDiff: (id: string, path: string, oldPath: string | null, status: FileStatus, options: DiffOptions) =>
    invoke<FileDiff>("working_file_diff", { id, path, oldPath, status, ignoreWhitespace: options.ignoreWhitespace }),
  conflictList: (id: string) => invoke<ConflictFile[]>("conflict_list_cmd", { id }),
  conflictDetail: (id: string, path: string) =>
    invoke<ConflictFileDetail>("conflict_detail_cmd", { id, path }),
  conflictResolveText: (id: string, path: string, content: string) =>
    invoke<RepoSnapshot>("conflict_resolve_text_cmd", { id, path, content }),
  conflictTakeStage: (id: string, path: string, stage: ConflictStageChoice) =>
    invoke<RepoSnapshot>("conflict_take_stage_cmd", { id, path, stage }),
  conflictDeletePath: (id: string, path: string) =>
    invoke<RepoSnapshot>("conflict_delete_path_cmd", { id, path }),
  commitFileDiff: (id: string, hash: string, path: string, oldPath: string | null, status: ChangedFileStatus, options: DiffOptions) =>
    invoke<FileDiff>("commit_file_diff", { id, hash, path, oldPath, status, ignoreWhitespace: options.ignoreWhitespace }),
  rollbackFile: (id: string, path: string, oldPath: string | null, status: FileStatus) =>
    invoke<RepoSnapshot>("rollback_file", { id, path, oldPath, status }),
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
  branchDelete: (id: string, name: string, kind: BranchKind, force: boolean) =>
    invoke<RepoSnapshot>("branch_delete", { id, name, kind, force }),
  branchDeletePrecheck: (id: string, name: string, kind: BranchKind) =>
    invoke<DeletePrecheck>("branch_delete_precheck", { id, name, kind }),
  commit: (id: string, files: string[], message: string, skipHooks: boolean) =>
    invoke<RepoSnapshot>("commit", { id, files, message, skipHooks }),
  merge: (id: string, branch: string) => invoke<RepoSnapshot>("merge", { id, branch }),
  mergeAbort: (id: string) => invoke<RepoSnapshot>("merge_abort_cmd", { id }),
  mergeContinue: (id: string) => invoke<RepoSnapshot>("merge_continue_cmd", { id }),
  rebaseAbort: (id: string) => invoke<RepoSnapshot>("rebase_abort_cmd", { id }),
  rebaseContinue: (id: string) => invoke<RepoSnapshot>("rebase_continue_cmd", { id }),
  cherryPickAbort: (id: string) => invoke<RepoSnapshot>("cherry_pick_abort_cmd", { id }),
  cherryPickContinue: (id: string) => invoke<RepoSnapshot>("cherry_pick_continue_cmd", { id }),
  revertAbort: (id: string) => invoke<RepoSnapshot>("revert_abort_cmd", { id }),
  revertContinue: (id: string) => invoke<RepoSnapshot>("revert_continue_cmd", { id }),
};
