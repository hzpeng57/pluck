use crate::error::GitResult;
use crate::git::cmd::run_git;
use std::path::Path;

pub async fn fetch_all(repo: &Path) -> GitResult<()> {
    run_git(repo, &["fetch", "--all", "--prune"]).await?;
    Ok(())
}
