use crate::error::GitResult;
use crate::git::cmd::run_git;
use std::path::Path;

pub async fn push(repo: &Path, force_with_lease: bool) -> GitResult<()> {
    // Resolve current branch and use `-u` so first push sets upstream.
    let head = run_git(repo, &["symbolic-ref", "--short", "HEAD"]).await?;
    let branch = head.stdout.trim().to_string();
    let mut args: Vec<&str> = vec!["push", "-u", "origin", &branch];
    if force_with_lease { args.insert(1, "--force-with-lease"); }
    run_git(repo, &args).await?;
    Ok(())
}
