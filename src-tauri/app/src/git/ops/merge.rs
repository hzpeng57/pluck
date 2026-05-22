use crate::error::GitResult;
use crate::git::cmd::run_git;
use std::path::Path;

pub async fn merge_into_current(repo: &Path, branch: &str) -> GitResult<()> {
    run_git(repo, &["merge", "--no-edit", branch]).await?;
    Ok(())
}

pub async fn merge_abort(repo: &Path) -> GitResult<()> {
    run_git(repo, &["merge", "--abort"]).await?;
    Ok(())
}

pub async fn merge_continue(repo: &Path) -> GitResult<()> {
    run_git(repo, &["merge", "--continue"]).await?;
    Ok(())
}
