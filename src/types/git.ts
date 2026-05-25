export type FileStatus = "modified" | "added" | "deleted" | "renamed" | "untracked" | "conflicted";
export interface WorkingFile { path: string; oldPath: string | null; status: FileStatus }
export interface HeadInfo { branch: string | null; detachedAt: string | null }
export type BranchKind = "local" | "remote";
export interface Branch {
  name: string; kind: BranchKind; upstream: string | null;
  ahead: number; behind: number; isCurrent: boolean; lastCommitShort: string;
}
export interface BranchList { local: Branch[]; remote: Branch[] }
export interface Commit {
  hash: string; short: string; author: string; email: string; dateUnix: number;
  subject: string; body: string; parents: string[]; refs: string[];
}
export interface RemoteStatus { upstream: string | null; ahead: number; behind: number }
export type GitOp =
  | { type: "merging"; from: string }
  | { type: "rebasing"; onto: string; head: string }
  | { type: "cherryPicking" };
export interface RepoSnapshot {
  head: HeadInfo; files: WorkingFile[]; branches: BranchList; log: Commit[];
  remoteStatus: RemoteStatus; inProgress: GitOp | null;
}
export interface RepoMeta { id: string; path: string; name: string }
export type ChangedFileStatus = "added" | "modified" | "deleted" | "renamed" | "copied" | "typechange";
export interface ChangedFile { status: ChangedFileStatus; path: string; oldPath: string | null }
export interface CommitDetail {
  hash: string; short: string; author: string; email: string; dateUnix: number;
  subject: string; body: string; parents: string[]; refs: string[]; files: ChangedFile[];
}
