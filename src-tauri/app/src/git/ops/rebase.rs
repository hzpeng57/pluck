use crate::error::{GitError, GitResult};
use crate::git::cmd::{git_command, run_git, run_git_non_interactive};
use crate::git::git_dir;
use crate::git::ops::conflict::ensure_no_unresolved_conflicts;
use std::path::Path;

pub async fn rebase_interactive(
    repo: &Path,
    from_commit: &str,
    bridge_bin: &Path,
    sock: &Path,
) -> GitResult<bool> {
    let output = git_command(repo)
        .env("GIT_SEQUENCE_EDITOR", bridge_bin)
        .env("GIT_EDITOR", bridge_bin)
        .env("PLUCK_GIT_SOCK", sock)
        .args(["rebase", "-i", &format!("{from_commit}^")])
        .output()
        .await
        .map_err(|e| GitError::spawn(e.to_string()))?;
    if output.status.success() {
        return Ok(false);
    }
    // Non-zero exit covers two benign cases the UI should not toast about:
    //   1. User aborted via the editor bridge (no rebase state left on disk).
    //   2. Conflicts mid-rebase (InProgressBanner takes over via snapshot.inProgress).
    // In both cases the upcoming refresh_session call rebuilds the snapshot and
    // the UI tells the right story. Only surface genuine setup failures
    // (bad ref, unknown commit, etc.) where no rebase ever started.
    let git_dir = git_dir(repo);
    let rebase_state_present = git_dir.join("rebase-merge").exists()
        || git_dir.join("rebase-apply").exists();
    let stderr = String::from_utf8_lossy(&output.stderr);
    let editor_aborted = stderr.contains("problem with the editor");
    if rebase_state_present {
        return Ok(false);
    }
    if editor_aborted {
        return Ok(true);
    }
    Err(GitError::from_stderr(
        output.status.code().unwrap_or(-1),
        &stderr,
    ))
}

pub async fn rebase_continue(repo: &Path) -> GitResult<()> {
    ensure_no_unresolved_conflicts(repo).await?;
    run_git_non_interactive(repo, &["rebase", "--continue"]).await?;
    Ok(())
}

pub async fn rebase_abort(repo: &Path) -> GitResult<()> {
    run_git(repo, &["rebase", "--abort"]).await?;
    Ok(())
}
