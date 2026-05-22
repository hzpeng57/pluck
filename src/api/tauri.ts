import { invoke } from "@tauri-apps/api/core";
import type { RepoMeta, RepoSnapshot } from "../types/git";

export const api = {
  repoAdd: (path: string) => invoke<RepoMeta>("repo_add", { path }),
  repoOpen: (id: string) => invoke<RepoSnapshot>("repo_open", { id }),
  repoRefresh: (id: string, logBranch?: string) =>
    invoke<RepoSnapshot>("repo_refresh", { id, logBranch: logBranch ?? null }),
};

export const ops = {
  branchCheckout: (id: string, name: string) =>
    invoke<RepoSnapshot>("branch_checkout", { id, name }),
  branchCreate: (id: string, name: string, from: string | null) =>
    invoke<RepoSnapshot>("branch_create", { id, name, from }),
  branchDelete: (id: string, name: string, force: boolean) =>
    invoke<RepoSnapshot>("branch_delete", { id, name, force }),
};
