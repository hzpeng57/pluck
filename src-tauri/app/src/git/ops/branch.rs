use crate::error::GitResult;
use crate::git::cmd::run_git;
use std::path::Path;

pub async fn create_branch(repo: &Path, name: &str, from: Option<&str>) -> GitResult<()> {
    if let Some(from) = from {
        run_git(repo, &["checkout", "-b", name, from]).await?;
    } else {
        run_git(repo, &["checkout", "-b", name]).await?;
    }
    Ok(())
}

pub async fn delete_branch(repo: &Path, name: &str, force: bool) -> GitResult<()> {
    let flag = if force { "-D" } else { "-d" };
    run_git(repo, &["branch", flag, name]).await?;
    Ok(())
}
