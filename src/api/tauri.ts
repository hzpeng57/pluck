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
};
