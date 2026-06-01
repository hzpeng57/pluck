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
pub struct GitIdentity {
    pub name: String,
    pub email: String,
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
    pub me: GitIdentity,
}

const LOG_FORMAT: &str = "%H%x1f%h%x1f%an%x1f%ae%x1f%at%x1f%P%x1f%D%x1f%s%x1f%b%x1e";
const BRANCH_FORMAT: &str =
    "%(refname)%00%(HEAD)%00%(upstream:short)%00%(upstream:track)%00%(objectname:short)";

pub const LOG_PAGE_SIZE: u32 = 200;

pub async fn log_page(
    repo: &Path,
    log_target: &str,
    skip: u32,
    limit: u32,
) -> GitResult<Vec<Commit>> {
    let log_fmt = format!("--format={}", LOG_FORMAT);
    let skip_arg = format!("--skip={}", skip);
    let limit_arg = limit.to_string();
    let log_out = run_git(
        repo,
        &["log", &log_fmt, "-n", &limit_arg, &skip_arg, log_target],
    )
    .await?;
    parse_log(&log_out.stdout)
}

fn looks_like_hash_prefix(q: &str) -> bool {
    q.len() >= 4 && q.len() <= 40 && q.chars().all(|c| c.is_ascii_hexdigit())
}

/// 服务端搜索：一次性扫描整个 history。query 与 author 任一非空即生效，两者 AND 组合。
/// 1) 若 query 形似 hash 前缀（4-40 hex），先用 `git rev-parse` 单点解析，命中放结果开头
///    （hash 路径不受 author 限制——hash 是全局唯一标识）
/// 2) 用 `git log [--grep -i --fixed-strings] [--author -i] -n <limit> <branch>`
pub async fn log_search(
    repo: &Path,
    log_target: &str,
    query: &str,
    author: &str,
    limit: u32,
) -> GitResult<Vec<Commit>> {
    let log_fmt = format!("--format={}", LOG_FORMAT);
    let limit_arg = limit.to_string();

    let mut results: Vec<Commit> = Vec::new();
    let mut hash_hit: Option<String> = None;

    // 仅当 query 像 hash 才尝试单点解析；author-only 搜索没必要走这条
    if !query.is_empty() && looks_like_hash_prefix(query) {
        let spec = format!("{}^{{commit}}", query);
        if let Ok(out) = run_git(repo, &["rev-parse", "--verify", "--quiet", &spec]).await {
            let hash = out.stdout.trim().to_string();
            if !hash.is_empty() {
                let one = run_git(repo, &["log", &log_fmt, "-n", "1", &hash]).await?;
                let mut parsed = parse_log(&one.stdout)?;
                if let Some(c) = parsed.pop() {
                    hash_hit = Some(c.hash.clone());
                    results.push(c);
                }
            }
        }
    }

    // 动态拼参数：subject/body grep 与 author 二选一或同时
    let grep_arg = format!("--grep={}", query);
    let author_arg = format!("--author={}", author);
    let mut args: Vec<&str> = vec!["log", &log_fmt, "-i"];
    if !query.is_empty() {
        args.push("--fixed-strings");
        args.push(&grep_arg);
    }
    if !author.is_empty() {
        args.push(&author_arg);
    }
    args.push("-n");
    args.push(&limit_arg);
    args.push(log_target);

    let grep_out = run_git(repo, &args).await?;
    for c in parse_log(&grep_out.stdout)? {
        if hash_hit.as_deref() == Some(&c.hash) {
            continue;
        }
        results.push(c);
    }

    Ok(results)
}

fn branch_list_contains(branches: &BranchList, name: &str) -> bool {
    branches
        .local
        .iter()
        .chain(branches.remote.iter())
        .any(|b| b.name == name)
}

async fn read_identity(repo: &Path) -> GitIdentity {
    let name = run_git(repo, &["config", "user.name"])
        .await
        .map(|o| o.stdout.trim().to_string())
        .unwrap_or_default();
    let email = run_git(repo, &["config", "user.email"])
        .await
        .map(|o| o.stdout.trim().to_string())
        .unwrap_or_default();
    GitIdentity { name, email }
}

pub async fn build_snapshot(repo: &Path, log_branch: Option<&str>) -> GitResult<RepoSnapshot> {
    let branch_fmt = format!("--format={}", BRANCH_FORMAT);
    let branch_args: &[&str] = &["for-each-ref", &branch_fmt, "refs/heads", "refs/remotes"];
    let (status_out, branch_out, in_prog, me) = tokio::join!(
        run_git(repo, &["status", "--porcelain=v2", "--branch"]),
        run_git(repo, branch_args),
        detect_in_progress(repo),
        read_identity(repo),
    );

    let status = parse_status_porcelain_v2(&status_out?.stdout)?;
    let branches_flat = parse_branches(&branch_out?.stdout, status.head.branch.as_deref());
    let branches = split_branch_list(branches_flat);

    let log_target = log_branch
        .filter(|name| branch_list_contains(&branches, name))
        .or(status.head.branch.as_deref())
        .unwrap_or("HEAD");
    let log = log_page(repo, log_target, 0, LOG_PAGE_SIZE).await?;

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
        me,
    })
}
