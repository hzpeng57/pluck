use crate::error::GitResult;
use crate::git::cmd::run_git;
use std::path::Path;

pub async fn checkout_branch(repo: &Path, name: &str) -> GitResult<()> {
    run_git(repo, &["checkout", name]).await?;
    Ok(())
}
