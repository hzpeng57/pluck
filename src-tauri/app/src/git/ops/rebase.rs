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
    let output = Command::new("git")
        .current_dir(repo)
        .env("GIT_SEQUENCE_EDITOR", bridge_bin)
        .env("GIT_EDITOR", bridge_bin)
        .env("PLUCK_GIT_SOCK", sock)
        .args(["rebase", "-i", &format!("{from_commit}^")])
        .output()
        .await
        .map_err(|e| GitError::spawn(e.to_string()))?;
    if output.status.success() {
        return Ok(());
    }
    // Non-zero exit covers two benign cases the UI should not toast about:
    //   1. User aborted via the editor bridge (no rebase state left on disk).
    //   2. Conflicts mid-rebase (InProgressBanner takes over via snapshot.inProgress).
    // In both cases the upcoming refresh_session call rebuilds the snapshot and
    // the UI tells the right story. Only surface genuine setup failures
    // (bad ref, unknown commit, etc.) where no rebase ever started.
    let rebase_state_present = repo.join(".git/rebase-merge").exists()
        || repo.join(".git/rebase-apply").exists();
    let stderr = String::from_utf8_lossy(&output.stderr);
    let editor_aborted = stderr.contains("problem with the editor");
    if rebase_state_present || editor_aborted {
        return Ok(());
    }
    Err(GitError::from_stderr(
        output.status.code().unwrap_or(-1),
        &stderr,
    ))
}

pub async fn rebase_continue(repo: &Path) -> GitResult<()> {
    run_git(repo, &["rebase", "--continue"]).await?;
    Ok(())
}

pub async fn rebase_abort(repo: &Path) -> GitResult<()> {
    run_git(repo, &["rebase", "--abort"]).await?;
    Ok(())
}
