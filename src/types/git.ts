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
  | { type: "cherryPicking" }
  | { type: "reverting" };
export type ResetMode = "soft" | "mixed" | "hard" | "keep";
export interface GitIdentity { name: string; email: string }
export interface RepoSnapshot {
  head: HeadInfo; files: WorkingFile[]; branches: BranchList; log: Commit[];
  remoteStatus: RemoteStatus; inProgress: GitOp | null; me: GitIdentity;
}
export interface RepoMeta { id: string; path: string; name: string }
export type ChangedFileStatus = "added" | "modified" | "deleted" | "renamed" | "copied" | "typechange";
export interface ChangedFile { status: ChangedFileStatus; path: string; oldPath: string | null }
export interface CommitDetail {
  hash: string; short: string; author: string; email: string; dateUnix: number;
  subject: string; body: string; parents: string[]; refs: string[]; files: ChangedFile[];
}
export type DiffKind = "workingTree" | "commit";
export type DiffLineKind = "context" | "added" | "deleted" | "noNewline";
export interface DiffLine {
  kind: DiffLineKind;
  oldNumber: number | null;
  newNumber: number | null;
  content: string;
}
export interface DiffHunk {
  header: string;
  oldStart: number;
  oldLines: number;
  newStart: number;
  newLines: number;
  lines: DiffLine[];
}
export interface FileDiff {
  kind: DiffKind;
  path: string;
  oldPath: string | null;
  status: FileStatus | ChangedFileStatus | "copied" | "typechange";
  binary: boolean;
  tooLarge: boolean;
  additions: number;
  deletions: number;
  hunks: DiffHunk[];
}
export type DiffTarget =
  | { kind: "workingTree"; path: string; oldPath: string | null; status: FileStatus }
  | { kind: "commit"; hash: string; path: string; oldPath: string | null; status: ChangedFileStatus };
export interface DeletePrecheck {
  exists: boolean;
  isCurrent: boolean;
  isMerged: boolean;
  upstream: string | null;
  aheadOfHead: number;
}
