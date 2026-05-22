use crate::error::{GitError, GitResult};
use crate::git::cmd::run_git;
use std::path::Path;

pub async fn pull_rebase(repo: &Path, target_branch: &str) -> GitResult<()> {
    let head = run_git(repo, &["symbolic-ref", "--short", "HEAD"]).await?;
    let current = head.stdout.trim().to_string();
    if current == target_branch {
        run_git(repo, &["pull", "--rebase"]).await?;
        return Ok(());
    }
    // Non-current target: fetch-only fast-forward via refspec.
    // Find upstream of target branch
    let upstream = run_git(repo, &["rev-parse", "--abbrev-ref", &format!("{target_branch}@{{upstream}}")]).await
        .map_err(|_| GitError::parse(format!("{target_branch} has no upstream configured")))?;
    let upstream = upstream.stdout.trim().to_string();
    let (remote, _rb) = upstream.split_once('/').ok_or_else(|| GitError::parse("upstream not in remote/branch form"))?;
    run_git(repo, &["fetch", remote, &format!("{target_branch}:{target_branch}")]).await
        .map_err(|e| GitError::parse(format!("Non-fast-forward — checkout {target_branch} first and pull manually. ({e})")))?;
    Ok(())
}
