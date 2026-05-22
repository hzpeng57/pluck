use crate::error::GitResult;
use crate::git::{cmd::run_git, detect::detect_in_progress, detect::GitOp, parse::*};
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteStatus {
    pub upstream: Option<String>,
    pub ahead: u32,
    pub behind: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoSnapshot {
    pub head: HeadInfo,
    pub files: Vec<WorkingFile>,
    pub branches: BranchList,
    pub log: Vec<Commit>,
    pub remote_status: RemoteStatus,
    pub in_progress: Option<GitOp>,
}

const LOG_FORMAT: &str = "%H%x1f%h%x1f%an%x1f%ae%x1f%at%x1f%P%x1f%D%x1f%s%x1f%b%x1e";
const BRANCH_FORMAT: &str =
    "%(refname)%00%(HEAD)%00%(upstream:short)%00%(upstream:track)%00%(objectname:short)";

pub async fn build_snapshot(repo: &Path, log_branch: Option<&str>) -> GitResult<RepoSnapshot> {
    let branch_fmt = format!("--format={}", BRANCH_FORMAT);
    let branch_args: &[&str] = &["for-each-ref", &branch_fmt, "refs/heads", "refs/remotes"];
    let (status_out, branch_out, in_prog) = tokio::join!(
        run_git(repo, &["status", "--porcelain=v2", "--branch"]),
        run_git(repo, branch_args),
        detect_in_progress(repo),
    );

    let status = parse_status_porcelain_v2(&status_out?.stdout)?;
    let branches_flat = parse_branches(&branch_out?.stdout, status.head.branch.as_deref());
    let branches = split_branch_list(branches_flat);

    let log_target = log_branch
        .or(status.head.branch.as_deref())
        .unwrap_or("HEAD");
    let log_fmt = format!("--format={}", LOG_FORMAT);
    let log_out = run_git(repo, &["log", &log_fmt, "-n", "200", log_target]).await?;
    let log = parse_log(&log_out.stdout)?;

    Ok(RepoSnapshot {
        head: status.head,
        files: status.files,
        branches,
        log,
        remote_status: RemoteStatus {
            upstream: status.upstream,
            ahead: status.ahead,
            behind: status.behind,
        },
        in_progress: in_prog,
    })
}
