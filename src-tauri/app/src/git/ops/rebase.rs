use crate::error::{GitError, GitResult};
use crate::git::cmd::run_git;
use std::path::Path;

pub async fn rebase_interactive(
    repo: &Path,
    from_commit: &str,
    bridge_bin: &Path,
    sock: &Path,
) -> GitResult<()> {
    use tokio::process::Command;
    let status = Command::new("git")
        .current_dir(repo)
        .env("GIT_SEQUENCE_EDITOR", bridge_bin)
        .env("GIT_EDITOR", bridge_bin)
        .env("TTGIT_SOCK", sock)
        .args(["rebase", "-i", &format!("{from_commit}^")])
        .status()
        .await
        .map_err(|e| GitError::spawn(e.to_string()))?;
    if !status.success() {
        return Err(GitError::from_stderr(
            status.code().unwrap_or(-1),
            "rebase failed (likely conflicts)",
        ));
    }
    Ok(())
}

pub async fn rebase_continue(repo: &Path) -> GitResult<()> {
    run_git(repo, &["rebase", "--continue"]).await?;
    Ok(())
}

pub async fn rebase_abort(repo: &Path) -> GitResult<()> {
    run_git(repo, &["rebase", "--abort"]).await?;
    Ok(())
}
